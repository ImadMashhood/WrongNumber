"""Render the supplied design brief; never silently rewrite its source text."""
from pathlib import Path
import re
from html import escape
from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.pagesizes import A4
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, PageBreak, KeepTogether
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont

ROOT=Path(__file__).resolve().parents[2]
import os
FONT=Path(os.environ.get('WRONG_NUMBER_PDF_FONTS','/usr/share/fonts/liberation-sans-fonts'))
for name,file in [('Body','LiberationSans-Regular.ttf'),('Body-Bold','LiberationSans-Bold.ttf')]:
    pdfmetrics.registerFont(TTFont(name,str(FONT/file)))
pdfmetrics.registerFontFamily('Body',normal='Body',bold='Body-Bold',italic='Body',boldItalic='Body-Bold')
INK=colors.HexColor('#15303b');TEAL=colors.HexColor('#087d7c')
styles={
 'body':ParagraphStyle('Body',fontName='Body',fontSize=9.5,leading=14,spaceAfter=7,textColor=INK),
 'h1':ParagraphStyle('H1',fontName='Body-Bold',fontSize=17,leading=23,spaceBefore=17,spaceAfter=12,textColor=TEAL,keepWithNext=True),
 'h2':ParagraphStyle('H2',fontName='Body-Bold',fontSize=12,leading=17,spaceBefore=12,spaceAfter=8,textColor=INK,keepWithNext=True),
 'title':ParagraphStyle('Title',fontName='Body-Bold',fontSize=34,leading=42,spaceAfter=15,textColor=INK),
 'subtitle':ParagraphStyle('Subtitle',fontName='Body',fontSize=18,leading=26,spaceAfter=20,textColor=TEAL),
 'bullet':ParagraphStyle('Bullet',fontName='Body',fontSize=9.5,leading=14,leftIndent=14,firstLineIndent=-10,spaceAfter=5,textColor=INK),
}
def inline(value):
    value=escape(value.replace('–','-').replace('—','-').replace('\u2011','-'))
    return re.sub(r'\*\*(.+?)\*\*',r'<b>\1</b>',value)
def decorate(canvas,doc):
    canvas.saveState();canvas.setStrokeColor(TEAL);canvas.setLineWidth(.7);canvas.line(48,43,A4[0]-48,43)
    canvas.setFont('Body',8);canvas.setFillColor(INK);canvas.drawString(48,29,'WRONG NUMBER / GAMEPLAY DESIGN BRIEF');canvas.drawRightString(A4[0]-48,29,str(doc.page));canvas.restoreState()
source=(ROOT/'docs/design/gameplay-loop-source.txt').read_text()
blocks=source.strip().split('\n\n')
story=[Spacer(1,70),Paragraph('WRONG NUMBER',styles['title']),Paragraph('Complete Gameplay Loop<br/>&amp; Systems Specification',styles['subtitle'])]
for block in blocks[1:2]:story.append(Paragraph(inline(block).replace('\n','<br/>'),styles['body']))
story += [Spacer(1,40),Paragraph('User-supplied design brief / 27 October 2026',styles['h2']),Paragraph('Reference specification, not a list of completed features. The source text is preserved alongside this PDF. The supplied text ends mid-sentence in section 11.2; no missing requirements have been invented.',styles['body']),Paragraph('Implementation precedence: explicit user changes take priority. Keep chairs unnumbered, use E to pop into seats, keep daylight outside and retain simple low-poly graphics.',styles['body']),PageBreak()]
# Preserve all content after the document title/subtitle, including metadata.
for block in blocks[2:]:
    if block.strip()=='---':continue
    if block.startswith('# '):story.append(Paragraph(inline(block[2:]),styles['h1']))
    elif block.startswith('## '):story.append(Paragraph(inline(block[3:]),styles['h2']))
    elif all(line.startswith('- ') or re.match(r'^\d+\. ',line) for line in block.splitlines()):
        for line in block.splitlines():story.append(Paragraph(inline('• '+line[2:] if line.startswith('- ') else line),styles['bullet']))
    else:story.append(Paragraph(inline(block).replace('\n','<br/>'),styles['body']))
output=ROOT/'docs/design/gameplay-loop.pdf'
SimpleDocTemplate(str(output),pagesize=A4,rightMargin=48,leftMargin=48,topMargin=45,bottomMargin=58,title='Wrong Number - Complete Gameplay Loop & Systems Specification',author='Imad Mashhood').build(story,onFirstPage=decorate,onLaterPages=decorate)
print(output)
