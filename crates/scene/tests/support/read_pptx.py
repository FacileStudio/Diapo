import json
import sys

from pptx import Presentation
from pptx.util import Emu

deck = Presentation(sys.argv[1])
slides = []
for slide in deck.slides:
    shapes = list(slide.shapes)
    tables = [shape.table for shape in shapes if shape.has_table]
    slides.append(
        {
            "title": slide.shapes.title.text_frame.text if slide.shapes.title is not None else None,
            "text": [shape.text_frame.text for shape in shapes if shape.has_text_frame],
            "levels": [p.level for shape in shapes if shape.has_text_frame for p in shape.text_frame.paragraphs],
            "tables": [[[cell.text for cell in row.cells] for row in table.rows] for table in tables],
            "pictures": [shape.image.blob[:4] == b"\x89PNG" for shape in shapes if shape.shape_type == 13],
            "notes": slide.notes_slide.notes_text_frame.text if slide.has_notes_slide else None,
        }
    )
print(json.dumps({"width": Emu(deck.slide_width), "height": Emu(deck.slide_height), "slides": slides}))
