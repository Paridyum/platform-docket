#!/usr/bin/env bash
# Prove a cold spare across a real machine boundary (ADR-0072 D6, DECISION-DEBT D-53).
#
# WHAT THIS PROVES that tests/node.rs cannot: node B restores with no filesystem
# access to node A. Separate containers, separate volumes, the bundle arriving
# over a network as bytes. The unit tests share a kernel and a disk; the property
# under test here is that the spare needs nothing from the lost machine except
# the estate key -- and the key is supplied the way RUN-0004 supplies it, out of
# band, never copied from A.
#
# WHAT IT DOES NOT PROVE, and cannot: the Sync.com leg. RFC-0038 records that the
# vendor's client is Windows and Mac only, so it can never run in a Linux
# container. The `offhost` container is a stand-in that carries the same bytes
# over the same kind of hop. D-53 stays open for that one manual rehearsal.
set -euo pipefail

IMAGE=docket-node:rehearsal
NET=paridyum-rehearsal
pass=0; fail=0

cleanup() {
  docker rm -f node-a node-b node-c node-d offhost >/dev/null 2>&1 || true
  docker volume rm -f store-a store-b store-c store-d shared-store offhost-data >/dev/null 2>&1 || true
  docker network rm "$NET" >/dev/null 2>&1 || true
}
trap cleanup EXIT
cleanup

ok()   { printf '  \033[32mPASS\033[0m  %s\n' "$1"; pass=$((pass+1)); }
bad()  { printf '  \033[31mFAIL\033[0m  %s\n' "$1"; fail=$((fail+1)); }
check(){ if [ "$1" = "0" ]; then ok "$2"; else bad "$2"; fi; }

docker network create "$NET" >/dev/null
for v in store-a store-b store-c offhost-data shared-store; do docker volume create "$v" >/dev/null; done

# ── The estate key. ONE key, generated once, installed on each node the way
# RUN-0004 installs it: retrieved out of band, never copied between machines.
KEY=$(docker run --rm --entrypoint age-keygen "$IMAGE" 2>/dev/null)
WRONG=$(docker run --rm --entrypoint age-keygen "$IMAGE" 2>/dev/null)

run() { # run <container> <script>
  local name=$1; shift
  docker exec "$name" /bin/sh -c "$1"
}

start() { # start <name> <volume> <key>
  docker run -d --name "$1" --network "$NET" -v "$2":/paridyum/store \
    -e SOPS_AGE_KEY_FILE=/root/keys.txt -e DOCKET_OPERATOR=rohan \
    --entrypoint sleep "$IMAGE" 3600 >/dev/null
  printf '%s' "$3" | docker exec -i "$1" /bin/sh -c 'cat > /root/keys.txt && chmod 600 /root/keys.txt'
}

echo
echo "── node A: the machine that will be lost ──────────────────────────────"
start node-a store-a "$KEY"
for slug in vivaswan priyanshu aditya; do
  run node-a "docket --root /paridyum new $slug officer" >/dev/null
done
run node-a "docket --root /paridyum roster" | tail -1

echo
echo "── the durability gate ────────────────────────────────────────────────"
if run node-a "docket --root /paridyum store status" >/dev/null 2>&1; then
  bad "store status should exit non-zero while the register has no bundle"
else
  ok "store status refuses to call an unbacked register safe"
fi
run node-a "docket --root /paridyum vault backup" >/dev/null
run node-a "docket --root /paridyum store status" >/dev/null 2>&1
check $? "store status is clean once the bundle exists"

echo
echo "── the off-host store: only ciphertext leaves node A ──────────────────"
docker run -d --name offhost --network "$NET" -v offhost-data:/srv \
  --entrypoint sh "$IMAGE" -c 'cd /srv && python3 -m http.server 8000' >/dev/null
sleep 1
# Node A pushes the bundle. Streamed through a tar over the network -- node B
# never mounts node A's volume, and neither mounts the other's.
docker exec node-a tar -C /paridyum/store/vault -cf - _register \
  | docker exec -i offhost tar -C /srv -xf -
run offhost "ls /srv/_register"

plain=$(run offhost 'n=0; for f in /srv/_register/*; do \
  head -c 18 "$f" | grep -q "^age-encryption" || n=$((n+1)); done; echo $n' | tr -d ' ')
if [ "$plain" = "0" ]; then
  ok "every file that left node A is age ciphertext (ADR-0071 §1)"
else
  bad "$plain file(s) reached the off-host store as plaintext"
fi

echo
echo "── node A is destroyed ────────────────────────────────────────────────"
docker rm -f node-a >/dev/null && docker volume rm -f store-a >/dev/null
ok "node A and its store no longer exist"

echo
echo "── node B: a machine that has never seen the store ────────────────────"
start node-b store-b "$KEY"
run node-b "test ! -e /paridyum/store/register" && ok "node B starts with no register"
run node-b "cd /tmp && curl -sSf -O -J --remote-name-all \
   http://offhost:8000/_register/register-people.jsonl.age \
   http://offhost:8000/_register/register-events.jsonl.age \
 && mkdir -p /tmp/_register && mv /tmp/*.age /tmp/_register/" >/dev/null
run node-b "ls /tmp/_register" | tr '\n' ' '; echo

if run node-b "docket --root /paridyum store restore /tmp/_register /paridyum" ; then
  ok "node B restored and every chain verified"
else
  bad "the restore on node B did not verify"
fi

echo
echo "── the spare holds what the lost machine held ─────────────────────────"
got=$(run node-b "docket --root /paridyum roster" | grep -cE 'PAR-P-0000(01|02|03)')
if [ "$got" = "3" ]; then ok "all three people survived the loss"; else bad "only $got of 3 survived"; fi

next=$(run node-b "docket --root /paridyum new naman officer" | head -1 | awk '{print $1}')
if [ "$next" = "PAR-P-000004-4" ]; then
  ok "the sequence continues at $next rather than restarting (ADR-0051)"
else
  bad "the sequence restarted: got $next"
fi

echo
echo "── negative: the off-host store is useless without the key ───────────"
start node-c store-c "$WRONG"
run node-c "mkdir -p /tmp/_register && cd /tmp/_register && curl -sSf -O \
   http://offhost:8000/_register/register-people.jsonl.age -O \
   http://offhost:8000/_register/register-events.jsonl.age" >/dev/null
if run node-c "docket --root /paridyum store restore /tmp/_register /paridyum" >/dev/null 2>&1; then
  bad "a stranger's key opened the bundle"
else
  ok "a different estate key cannot open the bundle"
fi

echo
echo "── negative: two nodes on ONE store contend rather than diverge ───────"
# What happens if someone mounts a synced store into two places at once -- the
# mistake a cold spare exists to make unnecessary, and the one that mints
# duplicate identities if nothing refuses it.
docker rm -f node-b node-c >/dev/null
docker run -d --name node-b --network "$NET" -v shared-store:/paridyum/store \
  -e DOCKET_OPERATOR=rohan --entrypoint sleep "$IMAGE" 3600 >/dev/null
docker run -d --name node-c --network "$NET" -v shared-store:/paridyum/store \
  -e DOCKET_OPERATOR=aditya --entrypoint sleep "$IMAGE" 3600 >/dev/null
run node-b "docket --root /paridyum new first officer" >/dev/null
# Hold the lock on B, then try to write from C.
docker exec -d node-b /bin/sh -c 'docket --root /paridyum serve --port 8080'
sleep 1
if run node-c "docket --root /paridyum new second officer" >/dev/null 2>&1; then
  bad "a second node wrote a shared store while the first held it"
else
  ok "the second node was refused, naming the holder"
  run node-c "docket --root /paridyum new second officer 2>&1 | head -5" | sed 's/^/        /'
fi
dups=$(run node-c "docket --root /paridyum check 2>&1 | grep -c 'different people' || true" | tr -d ' ')
if [ "$dups" = "0" ]; then ok "no identifier was allocated twice"; else bad "$dups duplicated identifiers"; fi

echo
echo "── negative: a node refuses to be reachable from another machine ──────"
# NON-VACUOUS ORDER, and the first version of this check got it wrong: it curled
# a container that was not serving at all, so the failure proved only that
# nothing was listening. The server must be UP and answering on loopback before
# its unreachability from elsewhere means anything.
#
# On its OWN store, too. The first attempt ran it on node C, which shares a
# store with node B's running server -- so `serve` was refused by the lock, the
# API never came up, and the check reported a defect that was the lock working.
docker volume create store-d >/dev/null
docker run -d --name node-d --network "$NET" -v store-d:/paridyum/store \
  -e DOCKET_OPERATOR=rohan --entrypoint sleep "$IMAGE" 3600 >/dev/null
docker exec -d node-d /bin/sh -c 'docket --root /paridyum serve --port 8081'
sleep 2
if run node-d "curl -sSf --max-time 3 http://127.0.0.1:8081/api/health" >/dev/null 2>&1; then
  ok "node D's API answers on its own loopback"
  if run node-b "curl -sSf --max-time 3 http://node-d:8081/api/health" >/dev/null 2>&1; then
    bad "the same API was reachable from another container -- it has no auth"
  else
    ok "and is unreachable from node B (require_loopback)"
  fi
else
  bad "node D's API did not come up, so the reachability check proves nothing"
fi

echo
printf '\n%d passed, %d failed\n' "$pass" "$fail"
[ "$fail" = "0" ]
