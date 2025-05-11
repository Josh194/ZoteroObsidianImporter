import * as fs from 'fs';
import type { Archiver } from 'archiver';
import archiver from 'archiver';
import path from 'path';

pack();

function pack() {
	console.log("Beginning artifact pack");

	fs.mkdirSync(path.resolve(__dirname, './install'), { recursive: false });
	const output = fs.createWriteStream(path.resolve(__dirname, './install/zo-importer.xpi'));

	const archive: Archiver = archiver('zip', {
		zlib: { level: 9 }
	});

	archive.on('warning', function(err) {
		if (err.code === 'ENOENT') {
			console.log(err);
		} else {
			throw err;
		}
	});

	archive.on('error', function(err) {
		throw err;
	});

	archive.pipe(output);

	archive.file('./src/manifest.json', { name: 'manifest.json' });
	archive.directory('./build/', false);

	archive.finalize();

	console.log("Finished pack");
}