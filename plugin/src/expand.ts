import { Util } from "./util";

export async function expand_plugin() {
	let dir =  await Util.get_data_dir();

	switch (Util.get_os()) {
		case Util.OSType.Windows: {
			await Zotero.File.putContentsAsync(
				PathUtils.join(dir.path, "zo_select.bat"),
				"@echo off && cd \"%1%\" && \"./ZOImporter.exe\" select -f index.json -o select.json & pause\nexit %errorlevel%"
			);
			await Zotero.File.putContentsAsync(
				PathUtils.join(dir.path, "zo_import.bat"),
				"@echo off && cd \"%1%\" && \"./ZOImporter.exe\" import -f export.json & pause\nexit %errorlevel%"
			);

			break;
		}
		case Util.OSType.Mac: {
			await Zotero.File.putContentsAsync(
				PathUtils.join(dir.path, "zo_select.command"),
`osascript - "$1" <<EOF
	on run argv -- argv is a list of strings
		tell application "Terminal"
			activate
			do script (("cd ") & (quoted form of item 1 of argv) & (" && clear && ./ZOImporter select -f index.json -o select.json; err=\\$?; read -n 1 -s -r -p \\"Press any key to continue . . . \\"; echo \\"\\"; exit \\$err"))
		end tell
	end run
EOF`
			);
			await Zotero.File.putContentsAsync(
				PathUtils.join(dir.path, "zo_import.command"),
`osascript - "$1" <<EOF
	on run argv -- argv is a list of strings
		tell application "Terminal"
			activate
			do script (("cd ") & (quoted form of item 1 of argv) & (" && clear && ./ZOImporter import -f export.json; err=\\$?; read -n 1 -s -r -p \\"Press any key to continue . . . \\"; echo \\"\\"; exit \\$err"))
		end tell
	end run
EOF`
			);

			break;
		}
	}
}

export async function remove_plugin_expansion() {
	let dir = await Util.get_data_dir();

	switch (Util.get_os()) {
		case Util.OSType.Windows: {
			await Zotero.File.removeIfExists(PathUtils.join(dir.path, "zo_select.bat"));
			await Zotero.File.removeIfExists(PathUtils.join(dir.path, "zo_import.bat"));

			break;
		}
		case Util.OSType.Mac: {
			await Zotero.File.removeIfExists(PathUtils.join(dir.path, "zo_select.command"));
			await Zotero.File.removeIfExists(PathUtils.join(dir.path, "zo_import.command"));

			break;
		}
	}
}