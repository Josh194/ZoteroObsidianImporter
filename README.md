# Zotero-Obsidian Importer

**This project provides an opinionated and simple workflow for importing Zotero documents and their associated PDF annotations into a set of Obsidian notes.**

## Details

An Obsidian note is generated for each annotation, and for the document itself. The resulting notes carry over tags, highlight colors, some document metadata, document abstracts, and annotation text. Annotation notes link to their parent document note, and all notes contain a section for text that will not be overwritten on reimport (assuming the note still exists).

Notes are placed within collection-based folder paths.

This feature-set is incomplete, and may be strongly expanded on later.

## Architecture

The import system is comprised of a small Zotero plugin that provides an internal interface for retrieving document information (this will likely be partially or entirely replaced by the web API once the local implementation supports all features necessary), and a separate CLI importer program. The importer is invoked by the plugin through a right click menu temporarily located in the PDF reader.

## Notes

This project is currently in an alpha state; it does work fairly reliably if you know its limitations, but there are many sharp edges that still need to be smoothed out. Some of these include:
- Due to platform-specific issues related to invoking the importer, an internal shim program is used to run it indirectly. On windows, this can result in an extra terminal window being shown during importing.
- To deal with terminal CLI differences between platforms, the plugin currently has an undeclared dependency on [alacritty](https://github.com/alacritty/alacritty), which must be visible within the plugin's data directory (currently `<ZOTERO>/zo_import`).
- There is currently an extremely annoying bug where the plugin can be sporadically removed from Zotero for unknown reasons. To avoid the loss of configuration data, the plugin does not remove its data directory when it is uninstalled. In the event of the aforementioned removal, the plugin may be safely reinstalled without any further action.
- Zotero plugins must contain an auto-update link, but as this project is nowhere near stable enough to be thinking about auto-updating, the link currently points to the official Zotero example plugin. Auto updating should be manually disabled.
- The config file format is undocumented (as with essentially everything else). That being said, the current format can be determined by looking at `ProgramConfig` in `src/main.rs` (note that this structure is nested inside a JSON structure specifying a schema version).
- Persistent section handling in notes is currently somewhat unstable, and should not be relied on. In particular, only a single persistent section is supported, and must be present.
- Many errors are poorly reported. That being said, the importer will almost always error in response to unexpected input as opposed to silently breaking.
- Annotation notes currently have fairly useless and excessively long names. This is currently being worked on.

Finally, this project is by no means in a stable state yet, and every version should be expected to contain potentially breaking changes, and should not be expected to use the same configuration format.
Releases (and binary distributions) are currently not provided, but will be soon once I deem the external interface stable enough.