import * as fs from 'fs';
import type { Archiver } from 'archiver';
import archiver from 'archiver';

pack();

function pack() {
	console.log("Beginning artifact pack");

	const output = fs.createWriteStream(__dirname + '/install/zo-importer.xpi');

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