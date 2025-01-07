#!/usr/bin/env perl
# 先頭行は、Linux用のシバンなので、Windowsでは削除しても構いません。

# 通常のLaTeXドキュメント用のコマンド
# 今回はupLaTeXを設定してあります。
$latex = 'uplatex %O -kanji=utf8 -no-guess-input-enc -synctex=1 -interaction=nonstopmode %S';
# pdfLaTeX 用のコマンド 
$pdflatex = 'pdflatex %O -synctex=1 -interaction=nonstopmode %S';
# LuaLaTeX 用のコマンド 
$lualatex = 'lualatex %O -synctex=1 -interaction=nonstopmode %S';
# XeLaTeX 用のコマンド
$xelatex = 'xelatex %O -no-pdf -synctex=1 -shell-escape -interaction=nonstopmode %S';
# Biber, BibTeX 用のコマンド
$biber = 'biber %O --bblencoding=utf8 -u -U --output_safechars %B';
$bibtex = 'upbibtex %O %B';
# makeindex 用のコマンド
$makeindex = 'upmendex %O -o %D %S';
# dvipdf のコマンド
$dvipdf = 'dvipdfmx %O -o %D %S';
# dvips のコマンド
$dvips = 'dvips %O -z -f %S | convbkmk -u > %D';
$ps2pdf = 'ps2pdf.exe %O %S %D';
  
# $pdf_mode ...PDF の作成方法を指定するオプション 
# 0: $latex で .tex -> .dvi するだけ
# 1: $pdflatex で .tex -> .pdf (pdflatexは英文にしか使えない)
# 2: $latex で .tex -> .dvi / $dvips で .dvi -> .ps / $ps2pdf で .ps -> PDF
# 3: $latex で .tex -> .dvi / $dvipdf で .dvi -> PDF 
# 4: $lualatex で .tex -> PDF
# 5: $xelatex で .tex -> .xdv / $xdvipdfmx で .xdv -> PDF
# lualatexしか使わないなら以下行のコメント（#）を外すが、他も使いそうならこのまま
# $pdf_mode = 4; 
  
# PDFビューワの設定 
# "start %S": .pdf の規定のソフトで表示（Windowsのみ）
# Linuxの場合、"evince %S" を指定してください
# $pdf_previewer = "start %S";
