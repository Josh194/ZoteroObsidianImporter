import { Util } from "./util";

export async function expand_plugin() {
	let dir =  await Util.get_data_dir();

	switch (Util.get_os()) {
		case Util.OSType.Windows: {
			break;
		}
		case Util.OSType.Mac: {
			break;
		}
	}
}

export async function remove_plugin_expansion() {
	let dir = await Util.get_data_dir();

	switch (Util.get_os()) {
		case Util.OSType.Windows: {
			break;
		}
		case Util.OSType.Mac: {
			break;
		}
	}
}