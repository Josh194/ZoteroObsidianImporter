import webpack, { WebpackPluginInstance, Compiler, type Compilation } from "webpack";
const { sources } = webpack;

import archiver, { type Archiver } from "archiver";
import streamConsumers from "node:stream/consumers";

import fs from "fs";

class ZoteroPackError extends Error {
	constructor(message: string) {
		super(`[zotero-pack] ${message}`);
		this.name = ZoteroPackError.name;
		this.stack = "";
	}
}

export default class ZoteroPackPlugin implements WebpackPluginInstance {
	archive: string;
	manifest: string;

	constructor(name: string, manifest: string) {
		this.archive = name;
		this.manifest = manifest;
	}

	apply(compiler: Compiler): void {;
		compiler.hooks.thisCompilation.tap(ZoteroPackPlugin.name, compilation => {
			compilation.hooks.processAssets.tapPromise(
				{
					name: ZoteroPackPlugin.name,
					stage: webpack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_TRANSFER
				},
				() => this.pack(compilation)
			);
		});
	}

	async pack(compilation: Compilation): Promise<void> {
		const logger = compilation.getLogger(ZoteroPackPlugin.name);

		const archive: Archiver = archiver("zip", {
			zlib: { level: 9 }
		});

		if (!fs.existsSync(this.manifest)) {
			throw new ZoteroPackError("Manifest file does not exist");
		}

		archive.on("warning", function(err) {
			throw err;
		});

		archive.on("error", function(err) {
			throw err;
		});

		archive.file(this.manifest, { name: "manifest.json" });

		const entries = compilation.entrypoints;
		let fileCount: number = 0;

		for (let [name, entry] of entries) {
			let files = entry.getFiles();
			fileCount += files.length;

			for (let file of files) {
				let asset = compilation.assets[file];
				if (asset == undefined) { throw new ZoteroPackError(`Unable to find asset for file ${file}`); }

				const source = asset.source();

				archive.append(
					Buffer.isBuffer(source) ? source : Buffer.from(source),
					{ name: file }
				);
			}
		}

		logger.log(`${fileCount} files processed from ${entries.size} entrypoints`);

		await archive.finalize();

		const buffer = Buffer.from(await streamConsumers.arrayBuffer(archive));

		compilation.emitAsset(
			this.archive + ".xpi",
			new sources.RawSource(buffer),
			{ size: buffer.byteLength, javascriptModule: false }
		);
	}
}