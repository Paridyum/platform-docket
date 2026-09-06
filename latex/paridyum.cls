% ==================================================================
% paridyum.cls — the Paridyum official document class
%
% This file is the ONLY place document layout is defined. The header,
% address block, boxes and footer that were copy-pasted into eleven
% separate .tex files now live here once.
%
% Facts about the organisation (address, email, officers) are NOT
% hard-coded here. They arrive as \newcommand definitions generated
% from registry/org.yaml, and the \provide... defaults below exist
% only so a template can be compiled standalone while drafting.
% ==================================================================

\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{paridyum}[2026/08/19 v0.1 Paridyum official document]

% ---- Class options ------------------------------------------------
% draft — stamps the page DRAFT and suppresses the reference number,
%         so an unissued document can never be mistaken for an issued
%         one. `docket render` uses this; `docket issue` does not.
\newif\ifparidyum@draft
\DeclareOption{draft}{\paridyum@drafttrue}
\DeclareOption*{\PassOptionsToClass{\CurrentOption}{article}}
\ExecuteOptions{}
\ProcessOptions\relax

\LoadClass[12pt,a4paper]{article}

% ---- Packages -----------------------------------------------------
\RequirePackage[margin=1in]{geometry}
% [final] because the class's own `draft` option becomes a GLOBAL option and
% graphicx would otherwise substitute a box containing the filename for every
% image. Our `draft` means "not issued", not LaTeX's draft mode -- a draft
% must look like what will actually be issued.
\RequirePackage[final]{graphicx}
\RequirePackage{array,tabularx}
\RequirePackage{setspace}
\RequirePackage{parskip}
\RequirePackage{titlesec}
% PDF/A-2b via pdfx (ISO 19005-2), NOT via \DocumentMetadata.
%
% \DocumentMetadata is the modern route and it was tried first. In TeX Live
% 2026 it unconditionally loads the new tagging/block modules, whose list
% implementation rejects enumitem's keys -- every `\begin{itemize}[leftmargin=*]`
% in these templates fails with "Some keys specified on the itemize
% environment are unknown", and `tagging=off` does not prevent the modules
% loading. pdfx has no such conflict.
%
% pdfx MUST be loaded before hyperref and loads hyperref itself, which is why
% this replaces the hyperref line rather than sitting beside it. It reads
% document metadata from <jobname>.xmpdata, written by the tool at issue time.
\IfFileExists{pdfx.sty}{%
  \RequirePackage[a-2b]{pdfx}%
  \hypersetup{hidelinks}%
}{%
  \ClassWarning{paridyum}{pdfx not installed; output will not be PDF/A}%
  \RequirePackage[hidelinks]{hyperref}%
}
\RequirePackage[table]{xcolor}
\RequirePackage{tcolorbox}
\RequirePackage{enumitem}
\RequirePackage{amssymb}      % checkboxes on the forms
\RequirePackage{tikz}         % logo fallback, draft stamp
\RequirePackage{eso-pic}
\RequirePackage{fancyhdr}
\RequirePackage{lastpage}

% ---- Reproducible output ------------------------------------------
% pdfTeX derives the trailer /ID from the wall clock, which is the ONLY
% remaining source of non-determinism once SOURCE_DATE_EPOCH pins the
% dates. Without this, two builds of an identical source differ by 64
% bytes and no content hash in the register would ever be stable.
\ifdefined\pdftrailerid \pdftrailerid{paridyum-docket} \fi

% ---- General formatting -------------------------------------------
\setstretch{1.10}
\setlength{\parindent}{0pt}
\pagestyle{empty}

\definecolor{linegray}{HTML}{CFCFCF}
\definecolor{headgray}{HTML}{F4F4F4}
\definecolor{draftgray}{HTML}{D8D8D8}

% Every document in the estate uses \section*, so the numbered form was
% never reachable. Formatting the starred form is what was actually meant.
\titleformat{name=\section,numberless}
    {\bfseries\large}
    {}
    {0pt}
    {}
\titlespacing*{name=\section,numberless}{0pt}{1.1em}{0.4em}

\renewcommand{\arraystretch}{1.3}

% ---- Organisation facts -------------------------------------------
% NOTHING about the organisation is hard-coded in this class or in any
% template. Every value below is generated into data.tex from registry/
% by tools/gendata.py and injected before \documentclass takes effect.
%
% The \providecommand defaults exist for ONE reason: so a template can be
% compiled standalone while it is being written. A real build always
% overrides them. If you find yourself editing a value here, the value
% belongs in registry/org.yaml instead.
\providecommand{\dkOrgName}{PARIDYUM}
\providecommand{\dkTagline}{Technology \& Software}
\providecommand{\dkAddress}{[address not set]}
\providecommand{\dkEmail}{[email not set]}
\providecommand{\dkWeb}{[web not set]}
\providecommand{\dkLegalStatus}{[legal status not set]}
\providecommand{\dkLogoFile}{paridyum-logo}

% ---- Document reference -------------------------------------------
% Stored when the date/reference line is emitted so that every page can
% carry it. A multi-page instrument whose pages cannot be tied back to
% one reference is a document someone can interleave a page into.
\newcommand{\paridyum@ref}{}
\newcommand{\paridyumsetref}[1]{\renewcommand{\paridyum@ref}{#1}}

% Verification code, printed in the footer of every page of a document that
% the register can confirm. In the footer rather than the body because a
% certificate must fit on one page and a body block cannot promise that.
\newcommand{\paridyum@verify}{}
\newcommand{\paridyumverifiable}[1]{\renewcommand{\paridyum@verify}{#1}}

% ---- Boxes --------------------------------------------------------
\newtcolorbox{infobox}{
    colback=white, colframe=black, boxrule=0.5pt, arc=0pt,
    left=8pt, right=8pt, top=8pt, bottom=8pt
}

\newtcolorbox{softbox}{
    colback=headgray, colframe=black, boxrule=0.4pt, arc=0pt,
    left=8pt, right=8pt, top=6pt, bottom=6pt
}

% ---- Logo ---------------------------------------------------------
% Degrades to a drawn placeholder rather than failing the build when the
% asset is absent. A missing logo must be visible on the page, not a
% compile error that tempts someone to comment the header out.
\newcommand{\paridyum@logo}{%
  \IfFileExists{\dkLogoFile.pdf}{\includegraphics[width=1.9cm]{\dkLogoFile.pdf}}{%
  \IfFileExists{\dkLogoFile.png}{\includegraphics[width=1.9cm]{\dkLogoFile.png}}{%
    \begin{tikzpicture}
      \draw[draftgray,line width=0.4pt] (0,0) rectangle (1.9,1.9);
      \node[draftgray,font=\tiny] at (0.95,0.95) {logo};
    \end{tikzpicture}}}%
}

% ---- Letterhead ---------------------------------------------------
\newcommand{\paridyumletterhead}{%
  \thispagestyle{paridyumfirst}%
  \setlength{\tabcolsep}{0pt}%
  \begin{tabular}{@{}m{2.4cm}@{\hspace{0.45cm}}m{14.1cm}@{}}
    \centering \paridyum@logo
    &
    \begin{flushleft}
      {\fontsize{23}{25}\selectfont\textbf{\dkOrgName}}\\[2pt]
      {\normalsize\textit{\dkTagline}}\\[6pt]
      {\small \dkAddress}\\[3pt]
      {\small \dkEmail \quad\textbar\quad \dkWeb}
    \end{flushleft}
  \end{tabular}

  \vspace{0.65em}
  \noindent\rule{\textwidth}{0.5pt}
  \vspace{0.9em}
}

% ---- Date / reference line ----------------------------------------
% #1 date, #2 reference number. In draft mode the reference is replaced
% by a visible marker: an unissued document has no serial, and printing
% a placeholder that looks like one is how fake serials get into the world.
\newcommand{\paridyumdateref}[2]{%
  \paridyumsetref{#2}%
  \begin{tabularx}{\textwidth}{@{}Xr@{}}
    \textbf{Date:} #1 &
    \ifparidyum@draft\textbf{Ref. No.:} \textcolor{red}{NOT ISSUED}\else\textbf{Ref. No.:} #2\fi
  \end{tabularx}
  \vspace{0.9em}
}

% Date line with no reference number, for documents that are collected
% rather than issued (the forms).
\newcommand{\paridyumdate}[1]{%
  \begin{tabularx}{\textwidth}{@{}Xr@{}}
    \textbf{Date:} #1 &
  \end{tabularx}
  \vspace{0.9em}
}

% ---- Document title -----------------------------------------------
\newcommand{\paridyumtitle}[1]{%
  \begin{softbox}\begin{center}\textbf{\Large #1}\end{center}\end{softbox}
  \vspace{0.6em}
}

% ---- Signature primitives -----------------------------------------
% #1 name, #2 designation
\newcommand{\paridyumsignatory}[2]{%
  \rule{6cm}{0.4pt}\\ Name: #1 \\ Designation: #2 \\ Date: \rule{4cm}{0.4pt}%
}

% Two-officer block, used by every agreement and policy.
% #1 name1, #2 title1, #3 name2, #4 title2
\newcommand{\paridyumofficers}[4]{%
  \textbf{For \dkOrgName}

  \vspace{0.5em}
  \begin{tabularx}{\textwidth}{@{}X X@{}}
    \textbf{#2} & \textbf{#4} \\[2.5em]
    \rule{6cm}{0.4pt} & \rule{6cm}{0.4pt} \\
    Name: #1 & Name: #3 \\
    Date: \rule{4cm}{0.4pt} & Date: \rule{4cm}{0.4pt}
  \end{tabularx}
}

% Signature block resolved from the document rather than named in the template.
%
% WHO SIGNS DEPENDS ON WHO THE DOCUMENT IS ABOUT. An intern's NDA is signed by the
% CEO and CTO; an officer's is signed by the Director, so that nobody signs their own
% document (RFC-0042 decision 4). `docket` emits \dkSignerOne* always and \dkSignerTwo*
% only where the document has a second signatory, and this renders whichever it got.
% The alternative was a second copy of four templates differing in one line.
\newcommand{\paridyumsignblock}{%
  \ifcsname dkSignerTwoName\endcsname
    \paridyumofficers{\dkSignerOneName}{\dkSignerOneDesignation}%
                     {\dkSignerTwoName}{\dkSignerTwoDesignation}%
  \else
    \textbf{For \dkOrgName}

    \vspace{0.5em}
    \begin{tabularx}{\textwidth}{@{}X X@{}}
      \textbf{\dkSignerOneDesignation} & \\[2.5em]
      \rule{6cm}{0.4pt} & \\
      Name: \dkSignerOneName & \\
      Date: \rule{4cm}{0.4pt} &
    \end{tabularx}
  \fi
}

% Counterparty signature block. #1 role label, #2 name
\newcommand{\paridyumcountersign}[2]{%
  \textbf{#1}

  \vspace{0.5em}
  \begin{tabularx}{\textwidth}{@{}X X@{}}
    \rule{6cm}{0.4pt} & \\
    Name: #2 & \\
    Date: \rule{4cm}{0.4pt} &
  \end{tabularx}
}

% ---- Form fields --------------------------------------------------
% \dkfill{command} prints that value if the data defines it, and a
% fill-in rule if it does not. One template therefore serves both a blank
% form handed to an intern and a pre-filled one generated from the record
% -- nobody retypes what Paridyum already holds.
\newcommand{\dkfill}[2][7cm]{%
  \ifcsname #2\endcsname\csname #2\endcsname\else\rule{#1}{0.4pt}\fi}

% Always blank, regardless of data: signatures and hand-dated fields.
\newcommand{\dkblank}[1][7cm]{\rule{#1}{0.4pt}}

% Bold, ragged-right label column of a given width. A column type rather
% than a wrapper environment: tabularx scans for a literal \end{tabularx}
% and cannot be wrapped.
%   \begin{tabularx}{\linewidth}{@{}F{4.4cm}X@{}}
\newcolumntype{F}[1]{>{\bfseries\raggedright\arraybackslash}p{#1}}

% ---- Footer -------------------------------------------------------
% On EVERY page, not just the last. Carries "Page X of Y" and the
% reference number, so a page cannot be removed or inserted unnoticed.
% \parbox[b] anchors the block by its LAST baseline, so it grows upward
% from the footer line instead of off the bottom of the paper. With the
% default downward flow the "Page X of Y" line landed at 845.8pt on an
% 845.0pt page and was silently dropped.
\newcommand{\paridyum@footerblock}{%
  \parbox[b]{\textwidth}{%
    \centering
    \rule{\textwidth}{0.5pt}\\[0.35em]
    {\small \textbf{\dkOrgName} \quad | \quad \dkTagline \quad | \quad \dkLegalStatus}\\[0.25em]
    {\scriptsize Page \thepage\ of \pageref{LastPage}%
      \ifx\paridyum@ref\@empty\else\quad\textbar\quad Ref. \paridyum@ref\fi}%
    % One line, deliberately. The footer block grows upward from a fixed
    % baseline, so a line that wraps pushes the overflow off the page.
    \ifx\paridyum@verify\@empty\else
      \\[0.15em]
      {\scriptsize Verify with \dkOrgVerificationContact{} quoting this reference and
       code \texttt{\paridyum@verify}}%
    \fi%
  }%
}

\fancypagestyle{paridyum}{%
  \fancyhf{}%
  \renewcommand{\headrulewidth}{0pt}%
  \renewcommand{\footrulewidth}{0pt}%
  \fancyfoot[C]{\paridyum@footerblock}%
  % Continuation pages carry a compact identifying header.
  \fancyhead[L]{\small\textbf{\dkOrgName}}%
  \fancyhead[R]{\small\ifx\paridyum@ref\@empty\else Ref. \paridyum@ref\fi}%
}

% Page 1 has the full letterhead, so it takes no running header.
\fancypagestyle{paridyumfirst}{%
  \fancyhf{}%
  \renewcommand{\headrulewidth}{0pt}%
  \renewcommand{\footrulewidth}{0pt}%
  \fancyfoot[C]{\paridyum@footerblock}%
}

\setlength{\footskip}{42pt}
\setlength{\headheight}{15pt}
\AtBeginDocument{\pagestyle{paridyum}}

% ---- Draft stamp --------------------------------------------------
\ifparidyum@draft
  \AddToShipoutPictureBG{%
    \AtPageCenter{%
      \makebox(0,0){\rotatebox{45}{\textcolor{draftgray}{\fontsize{80}{80}\selectfont\bfseries DRAFT}}}%
    }%
  }
\fi

% ---- Document metadata --------------------------------------------
% Without this a PDF pulled out of the store -- emailed, downloaded from a
% cloud folder -- is anonymous: /Title, /Author and /Subject are empty and
% nothing but the printed body says whose it is. The person identifier goes
% in /Subject rather than a name, for the same reason folders are named by
% identifier (ADR-0051): metadata is read by anyone who has the file.
\providecommand{\dkDocTitle}{}
\providecommand{\dkDocSubject}{}
\providecommand{\dkDocKeywords}{}
\AtBeginDocument{%
  \hypersetup{
    pdftitle={\dkDocTitle},
    pdfauthor={\dkOrgName},
    pdfsubject={\dkDocSubject},
    pdfkeywords={\dkDocKeywords},
    pdfcreator={tool-docket},
  }%
}

% ---- Generated data -----------------------------------------------
% Loaded LAST so it overrides the drafting defaults above. Values the
% templates need but data.tex does not define stay undefined, and the
% build fails with "Undefined control sequence" -- which is the point.
% A signed document must never carry a silently blank field.
\InputIfFileExists{dkdata.tex}{}{}

\endinput
