use std::path::{Path, PathBuf};
use std::env;

// Subset of files to give us the ability to call flite_text_to_wave and
// cst_wave_resample
const FILE_LIST: [&str; 48] = [
    "lang/cmulex/cmu_lex.c",
    "lang/cmulex/cmu_lex_data.c",
    "lang/cmulex/cmu_lts_model.c",
    "lang/cmulex/cmu_lts_rules.c",
    "lang/cmulex/cmu_lex_entries.c",
    "lang/cmulex/cmu_postlex.c",
    "src/audio/au_streaming.c",
    "src/cg/cst_cg.c",
    "src/cg/cst_spamf0.c",
    "src/cg/cst_mlsa.c",
    "src/cg/cst_mlpg.c",
    "src/cg/cst_cg_map.c",
    "src/cg/cst_cg_load_voice.c",
    "src/cg/cst_vc.c",
    "src/hrg/cst_ffeature.c",
    "src/hrg/cst_item.c",
    "src/hrg/cst_relation.c",
    "src/hrg/cst_utterance.c",
    "src/lexicon/cst_lexicon.c",
    "src/lexicon/cst_lts.c",
    "src/regex/cst_regex.c",
    "src/regex/regexp.c",
    "src/stats/cst_cart.c",
    "src/speech/cst_track.c",
    "src/speech/cst_wave.c",
    "src/speech/cst_wave_io.c",
    "src/speech/cst_lpcres.c",
    "src/speech/rateconv.c",
    "src/synth/cst_synth.c",
    "src/synth/cst_ffeatures.c",
    "src/synth/cst_phoneset.c",
    "src/synth/cst_voice.c",
    "src/synth/flite.c",
    "src/synth/cst_utt_utils.c",
    "src/utils/cst_alloc.c",
    "src/utils/cst_error.c",
    "src/utils/cst_string.c",
    "src/utils/cst_features.c",
    "src/utils/cst_file_stdio.c",
    "src/utils/cst_tokenstream.c",
    "src/utils/cst_val.c",
    "src/utils/cst_val_user.c",
    "src/utils/cst_val_const.c",
    "src/utils/cst_endian.c",
    "src/utils/cst_url.c",
    "src/utils/cst_socket.c",
    "src/wavesynth/cst_clunits.c",
    // We are not reading/writing files through flite in our rust bindings, so
    // having an unimplemented mmap that's common across all platforms should be
    // fine. If we find in the future that this is causing problems we can
    // detect the target os and build the appropriate files
    "src/utils/cst_mmap_none.c",
];

fn find_c_files(path: &Path) -> Vec<PathBuf> {

    let mut ret = Vec::new();

    for entry in path.read_dir().expect("Failed to read path") {
        let entry = entry.expect("Invalid entry");
        let entry_path = entry.path();
        if entry.metadata().unwrap().is_dir() {
            ret.extend(find_c_files(&entry_path));
        }
        else if let Some(ext) = entry_path.extension() {
            if ext == "c" {
                ret.push(entry_path);
            }
        }
    }

    return ret;
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let vendor_path = manifest_dir.join("vendor");

    println!("{:?}", find_c_files(&vendor_path.join("src")));

    cc::Build::new()
        .files(FILE_LIST.iter().map(|&p| vendor_path.join(p)).collect::<Vec<_>>())
        .files(find_c_files(&vendor_path.join("lang/cmu_us_rms")))
        .files(find_c_files(&vendor_path.join("lang/usenglish")))
        .file(manifest_dir.join("src/flite_voice_list.c"))
        .file(manifest_dir.join("src/flite_lang_list.c"))
        .include(vendor_path.join("include"))
        .include(vendor_path.join("lang/usenglish"))
        .include(vendor_path.join("lang/cmulex"))
        .warnings(false)
        .compile("flite");

    let bindings = bindgen::builder()
        .clang_arg(format!("-I{}", vendor_path.join("include").to_string_lossy()))
        .header(manifest_dir.join("bindings.h").to_string_lossy())
        .generate();

    let out_dir = env::var("OUT_DIR").unwrap();
    bindings.unwrap().write_to_file(format!("{}/flite.rs", out_dir)).unwrap();
}
