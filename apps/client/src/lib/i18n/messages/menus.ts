import type { Row } from '../row';

/** The stage's context menus, alignment, arrangement and the keyboard shortcut list. */
export const menus = {
	'menu.stage': ['Slide actions', 'Actions sur la diapositive', 'Acciones de la diapositiva', 'Folienaktionen'],
	'menu.edit': ['Edit the text', 'Modifier le texte', 'Editar el texto', 'Text bearbeiten'],
	'menu.cut': ['Cut', 'Couper', 'Cortar', 'Ausschneiden'],
	'menu.copy': ['Copy', 'Copier', 'Copiar', 'Kopieren'],
	'menu.paste': ['Paste', 'Coller', 'Pegar', 'Einfügen'],
	'menu.duplicate': ['Duplicate', 'Dupliquer', 'Duplicar', 'Duplizieren'],
	'menu.delete': ['Delete', 'Supprimer', 'Eliminar', 'Löschen'],
	'menu.forward': ['Bring forward', 'Avancer', 'Traer adelante', 'Eine Ebene nach vorn'],
	'menu.backward': ['Send backward', 'Reculer', 'Enviar atrás', 'Eine Ebene nach hinten'],
	'menu.centerSlide': [
		'Center on the slide',
		'Centrer sur la diapositive',
		'Centrar en la diapositiva',
		'Auf der Folie zentrieren'
	],
	'menu.arrange': ['Arrange', 'Organiser', 'Organizar', 'Anordnen'],
	'menu.alignDistribute': [
		'Align and distribute',
		'Aligner et répartir',
		'Alinear y distribuir',
		'Ausrichten und verteilen'
	],
	'menu.textAlign': ['Text alignment', 'Alignement du texte', 'Alineación del texto', 'Textausrichtung'],
	'menu.lock': ['Lock', 'Verrouiller', 'Bloquear', 'Sperren'],
	'menu.unlock': ['Unlock', 'Déverrouiller', 'Desbloquear', 'Entsperren'],
	'menu.stepLater': [
		'Reveal one step later',
		'Faire apparaître une étape plus tard',
		'Mostrar un paso más tarde',
		'Einen Schritt später zeigen'
	],
	'menu.stepEarlier': [
		'Reveal one step earlier',
		'Faire apparaître une étape plus tôt',
		'Mostrar un paso antes',
		'Einen Schritt früher zeigen'
	],
	'menu.insert': ['Insert {what}', 'Insérer : {what}', 'Insertar: {what}', '{what} einfügen'],
	'menu.media': ['Image or video…', 'Image ou vidéo…', 'Imagen o vídeo…', 'Bild oder Video…'],
	'menu.selectAll': ['Select all', 'Tout sélectionner', 'Seleccionar todo', 'Alles auswählen'],
	'menu.shortcuts': ['Keyboard shortcuts', 'Raccourcis clavier', 'Atajos de teclado', 'Tastenkürzel'],
	'arrange.left': ['Align left edges', 'Aligner à gauche', 'Alinear a la izquierda', 'Links ausrichten'],
	'arrange.center': [
		'Align horizontal centers',
		'Centrer horizontalement',
		'Centrar horizontalmente',
		'Horizontal zentrieren'
	],
	'arrange.right': ['Align right edges', 'Aligner à droite', 'Alinear a la derecha', 'Rechts ausrichten'],
	'arrange.top': ['Align top edges', 'Aligner en haut', 'Alinear arriba', 'Oben ausrichten'],
	'arrange.middle': ['Align vertical centers', 'Centrer verticalement', 'Centrar verticalmente', 'Vertikal zentrieren'],
	'arrange.bottom': ['Align bottom edges', 'Aligner en bas', 'Alinear abajo', 'Unten ausrichten'],
	'arrange.horizontal': [
		'Distribute horizontally',
		'Répartir horizontalement',
		'Distribuir horizontalmente',
		'Horizontal verteilen'
	],
	'arrange.vertical': ['Distribute vertically', 'Répartir verticalement', 'Distribuir verticalmente', 'Vertikal verteilen'],
	'align.justify': ['Justify', 'Justifier', 'Justificar', 'Blocksatz'],
	'valign.top': ['Top', 'Haut', 'Arriba', 'Oben'],
	'valign.middle': ['Middle', 'Milieu', 'Centro', 'Mitte'],
	'valign.bottom': ['Bottom', 'Bas', 'Abajo', 'Unten'],
	'stage.label': ['Slide', 'Diapositive', 'Diapositiva', 'Folie'],
	'stage.role': ['slide canvas', 'zone de la diapositive', 'lienzo de la diapositiva', 'Folienfläche'],
	'stage.empty': [
		'Empty slide. Double-click to add text, or right-click for more.',
		'Diapositive vide. Double-cliquez pour ajouter du texte, ou faites un clic droit pour plus.',
		'Diapositiva vacía. Haga doble clic para añadir texto, o clic derecho para más.',
		'Leere Folie. Doppelklicken, um Text hinzuzufügen, oder Rechtsklick für mehr.'
	],
	'tool.insertHint': ['Insert {what}', 'Insérer : {what}', 'Insertar: {what}', '{what} einfügen'],
	'inspect.place': ['Position and size', 'Position et taille', 'Posición y tamaño', 'Position und Größe'],
	'inspect.text': ['Text', 'Texte', 'Texto', 'Text'],
	'inspect.valign': ['Vertical alignment', 'Alignement vertical', 'Alineación vertical', 'Vertikale Ausrichtung'],
	'inspect.look': ['Appearance', 'Apparence', 'Apariencia', 'Aussehen'],
	'inspect.motion': ['Reveal', 'Apparition', 'Aparición', 'Einblendung'],
	'rail.up': ['Move up', 'Monter', 'Subir', 'Nach oben'],
	'rail.down': ['Move down', 'Descendre', 'Bajar', 'Nach unten'],
	'assets.menu': ['File actions', 'Actions sur le fichier', 'Acciones del archivo', 'Dateiaktionen'],
	'assets.place': [
		'Insert on the slide',
		'Insérer sur la diapositive',
		'Insertar en la diapositiva',
		'Auf der Folie einfügen'
	],
	'assets.remove': ['Delete', 'Supprimer', 'Eliminar', 'Löschen'],
	'keys.edit': ['Edit the selected text', 'Modifier le texte sélectionné', 'Editar el texto seleccionado', 'Ausgewählten Text bearbeiten'],
	'keys.leave': [
		'Stop editing, then clear the selection',
		'Quitter l’édition, puis désélectionner',
		'Salir de la edición, luego deseleccionar',
		'Bearbeitung beenden, dann Auswahl aufheben'
	],
	'keys.nudge': [
		'Nudge by 1 or 10 pixels',
		'Déplacer de 1 ou 10 pixels',
		'Mover 1 o 10 píxeles',
		'Um 1 oder 10 Pixel verschieben'
	],
	'keys.textAlign': [
		'Align text left, center, right, justify',
		'Aligner le texte à gauche, au centre, à droite, justifier',
		'Alinear texto a la izquierda, centro, derecha, justificar',
		'Text links, zentriert, rechts, Blocksatz'
	],
	'keys.rail': [
		'Move the focused slide up or down',
		'Monter ou descendre la diapositive active',
		'Subir o bajar la diapositiva enfocada',
		'Fokussierte Folie nach oben oder unten'
	],
	'keys.menu': ['Open the context menu', 'Ouvrir le menu contextuel', 'Abrir el menú contextual', 'Kontextmenü öffnen'],
	'keys.addText': [
		'Add a text box',
		'Ajouter une zone de texte',
		'Añadir un cuadro de texto',
		'Textfeld hinzufügen'
	],
	'keys.doubleClick': [
		'Double-click the slide',
		'Double-clic sur la diapositive',
		'Doble clic en la diapositiva',
		'Doppelklick auf die Folie'
	]
} satisfies Record<string, Row>;
