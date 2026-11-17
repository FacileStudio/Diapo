import type { Row } from '../row';

/** The slide editor's panels: the slide rail, the toolbar, the inspector and the speaker notes. */
export const slides = {
	'rail.title': ['Slides', 'Diapositives', 'Diapositivas', 'Folien'],
	'rail.count': [
		{ one: '{count} slide', other: '{count} slides' },
		{ one: '{count} diapositive', other: '{count} diapositives' },
		{ one: '{count} diapositiva', other: '{count} diapositivas' },
		{ one: '{count} Folie', other: '{count} Folien' }
	],
	'rail.slide': ['Slide {number}', 'Diapositive {number}', 'Diapositiva {number}', 'Folie {number}'],
	'rail.add': ['Add a slide', 'Ajouter une diapositive', 'Añadir una diapositiva', 'Folie hinzufügen'],
	'rail.addAfter': [
		'New slide after this one',
		'Nouvelle diapositive après celle-ci',
		'Nueva diapositiva después de esta',
		'Neue Folie danach'
	],
	'rail.duplicate': ['Duplicate', 'Dupliquer', 'Duplicar', 'Duplizieren'],
	'rail.delete': ['Delete', 'Supprimer', 'Eliminar', 'Löschen'],
	'rail.menu': ['Slide actions', 'Actions de la diapositive', 'Acciones de la diapositiva', 'Folienaktionen'],
	'tool.label': ['Slide tools', 'Outils de la diapositive', 'Herramientas de diapositiva', 'Folienwerkzeuge'],
	'tool.text': ['Text', 'Texte', 'Texto', 'Text'],
	'tool.rect': ['Rectangle', 'Rectangle', 'Rectángulo', 'Rechteck'],
	'tool.ellipse': ['Ellipse', 'Ellipse', 'Elipse', 'Ellipse'],
	'tool.diamond': ['Diamond', 'Losange', 'Rombo', 'Raute'],
	'tool.image': ['Image', 'Image', 'Imagen', 'Bild'],
	'tool.video': ['Video', 'Vidéo', 'Vídeo', 'Video'],
	'tool.files': ['Add a file…', 'Ajouter un fichier…', 'Añadir un archivo…', 'Datei hinzufügen…'],
	'tool.front': ['Bring to front', 'Mettre au premier plan', 'Traer al frente', 'In den Vordergrund'],
	'tool.back': ['Send to back', 'Mettre à l’arrière-plan', 'Enviar al fondo', 'In den Hintergrund'],
	'tool.undo': ['Undo', 'Annuler', 'Deshacer', 'Rückgängig'],
	'tool.redo': ['Redo', 'Rétablir', 'Rehacer', 'Wiederholen'],
	'tool.delete': ['Delete', 'Supprimer', 'Eliminar', 'Löschen'],
	'inspect.title': ['Properties', 'Propriétés', 'Propiedades', 'Eigenschaften'],
	'inspect.slide': ['Slide', 'Diapositive', 'Diapositiva', 'Folie'],
	'inspect.name': ['Name', 'Nom', 'Nombre', 'Name'],
	'inspect.layout': ['Layout', 'Mise en page', 'Diseño', 'Layout'],
	'inspect.layout.none': ['None', 'Aucune', 'Ninguno', 'Keines'],
	'inspect.count': [
		{ one: '{count} shape', other: '{count} shapes' },
		{ one: '{count} forme', other: '{count} formes' },
		{ one: '{count} forma', other: '{count} formas' },
		{ one: '{count} Form', other: '{count} Formen' }
	],
	'inspect.mixed': ['Mixed', 'Mixte', 'Mixto', 'Gemischt'],
	'inspect.x': ['X', 'X', 'X', 'X'],
	'inspect.y': ['Y', 'Y', 'Y', 'Y'],
	'inspect.w': ['Width', 'Largeur', 'Ancho', 'Breite'],
	'inspect.h': ['Height', 'Hauteur', 'Alto', 'Höhe'],
	'inspect.rotation': ['Rotation (°)', 'Rotation (°)', 'Rotación (°)', 'Drehung (°)'],
	'inspect.fill': ['Fill', 'Remplissage', 'Relleno', 'Füllung'],
	'inspect.stroke': ['Stroke', 'Contour', 'Contorno', 'Kontur'],
	'inspect.strokeWidth': ['Stroke width', 'Épaisseur du contour', 'Grosor del contorno', 'Konturstärke'],
	'inspect.size': ['Font size', 'Taille du texte', 'Tamaño del texto', 'Schriftgröße'],
	'inspect.align': ['Alignment', 'Alignement', 'Alineación', 'Ausrichtung'],
	'align.left': ['Left', 'Gauche', 'Izquierda', 'Links'],
	'align.center': ['Center', 'Centre', 'Centro', 'Mitte'],
	'align.right': ['Right', 'Droite', 'Derecha', 'Rechts'],
	'inspect.opacity': ['Opacity', 'Opacité', 'Opacidad', 'Deckkraft'],
	'inspect.step': ['Appears at step', 'Apparaît à l’étape', 'Aparece en el paso', 'Erscheint bei Schritt'],
	'inspect.lock': ['Locked', 'Verrouillé', 'Bloqueado', 'Gesperrt'],
	'notes.title': ['Speaker notes', 'Notes de l’orateur', 'Notas del orador', 'Referentennotizen'],
	'notes.show': ['Show the notes', 'Afficher les notes', 'Mostrar las notas', 'Notizen einblenden'],
	'notes.hide': ['Hide the notes', 'Masquer les notes', 'Ocultar las notas', 'Notizen ausblenden'],
	'notes.placeholder': [
		'Notes for this slide. They show in the presenter view, never on screen.',
		'Notes de cette diapositive. Elles s’affichent dans la vue de l’orateur, jamais à l’écran.',
		'Notas de esta diapositiva. Se muestran en la vista del orador, nunca en pantalla.',
		'Notizen zu dieser Folie. Sie erscheinen in der Referentenansicht, nie auf der Leinwand.'
	]
} satisfies Record<string, Row>;
