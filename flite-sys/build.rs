use std::env;
use std::path::PathBuf;

// Subset of files to give us the ability to call flite_text_to_wave and
// cst_wave_resample
const FILE_LIST: [&str; 144] = [
    "src/audio/auclient.c",
    "src/audio/auserver.c",
    "src/audio/audio.c",
    "src/audio/au_streaming.c",
    "src/audio/au_none.c",
    "src/utils/cst_alloc.c",
    "src/utils/cst_error.c",
    "src/utils/cst_string.c",
    "src/utils/cst_wchar.c",
    "src/utils/cst_tokenstream.c",
    "src/utils/cst_val.c",
    "src/utils/cst_features.c",
    "src/utils/cst_endian.c",
    "src/utils/cst_socket.c",
    "src/utils/cst_val_const.c",
    "src/utils/cst_val_user.c",
    "src/utils/cst_args.c",
    "src/utils/cst_url.c",
    "src/utils/cst_mmap_none.c",
    "src/utils/cst_file_stdio.c",
    "src/regex/cst_regex.c",
    "src/regex/regexp.c",
    "src/regex/regsub.c",
    "src/hrg/cst_utterance.c",
    "src/hrg/cst_relation.c",
    "src/hrg/cst_item.c",
    "src/hrg/cst_ffeature.c",
    "src/hrg/cst_rel_io.c",
    "src/stats/cst_cart.c",
    "src/stats/cst_viterbi.c",
    "src/stats/cst_ss.c",
    "src/speech/cst_wave.c",
    "src/speech/cst_wave_io.c",
    "src/speech/cst_track.c",
    "src/speech/cst_track_io.c",
    "src/speech/cst_wave_utils.c",
    "src/speech/cst_lpcres.c",
    "src/speech/rateconv.c",
    "src/speech/g721.c",
    "src/speech/g72x.c",
    "src/speech/g723_24.c",
    "src/speech/g723_40.c",
    "src/lexicon/cst_lexicon.c",
    "src/lexicon/cst_lts.c",
    "src/lexicon/cst_lts_rewrites.c",
    "src/synth/cst_synth.c",
    "src/synth/cst_utt_utils.c",
    "src/synth/cst_voice.c",
    "src/synth/cst_phoneset.c",
    "src/synth/cst_ffeatures.c",
    "src/synth/cst_ssml.c",
    "src/synth/flite.c",
    "src/wavesynth/cst_units.c",
    "src/wavesynth/cst_clunits.c",
    "src/wavesynth/cst_diphone.c",
    "src/wavesynth/cst_sigpr.c",
    "src/wavesynth/cst_sts.c",
    "src/wavesynth/cst_reflpc.c",
    "src/cg/cst_cg.c",
    "src/cg/cst_mlsa.c",
    "src/cg/cst_mlpg.c",
    "src/cg/cst_vc.c",
    "src/cg/cst_cg_load_voice.c",
    "src/cg/cst_cg_dump_voice.c",
    "src/cg/cst_cg_map.c",
    "src/cg/cst_spamf0.c",
    "lang/cmulex/cmu_lts_rules.c",
    "lang/cmulex/cmu_lts_model.c",
    "lang/cmulex/cmu_lex.c",
    "lang/cmulex/cmu_lex_entries.c",
    "lang/cmulex/cmu_lex_data.c",
    "lang/cmulex/cmu_postlex.c",
    "lang/cmu_indic_lex/cmu_indic_lex.c",
    "lang/cmu_grapheme_lex/cmu_grapheme_lex.c",
    "lang/cmu_grapheme_lex/grapheme_unitran_tables.c",
    "lang/usenglish/us_int_accent_cart.c",
    "lang/usenglish/us_int_tone_cart.c",
    "lang/usenglish/us_f0_model.c",
    "lang/usenglish/us_dur_stats.c",
    "lang/usenglish/us_durz_cart.c",
    "lang/usenglish/us_f0lr.c",
    "lang/usenglish/us_phoneset.c",
    "lang/usenglish/us_ffeatures.c",
    "lang/usenglish/us_phrasing_cart.c",
    "lang/usenglish/us_gpos.c",
    "lang/usenglish/us_text.c",
    "lang/usenglish/us_expand.c",
    "lang/usenglish/us_nums_cart.c",
    "lang/usenglish/us_aswd.c",
    "lang/usenglish/usenglish.c",
    "lang/usenglish/us_pos_cart.c",
    "lang/cmu_indic_lang/cmu_indic_lang.c",
    "lang/cmu_indic_lang/cmu_indic_phoneset.c",
    "lang/cmu_indic_lang/cmu_indic_phrasing_cart.c",
    "lang/cmu_grapheme_lang/cmu_grapheme_lang.c",
    "lang/cmu_grapheme_lang/cmu_grapheme_phoneset.c",
    "lang/cmu_grapheme_lang/cmu_grapheme_phrasing_cart.c",
    "lang/cmu_us_kal/cmu_us_kal_diphone.c",
    "lang/cmu_us_kal/cmu_us_kal.c",
    "lang/cmu_us_kal/cmu_us_kal_lpc.c",
    "lang/cmu_us_kal/cmu_us_kal_res.c",
    "lang/cmu_us_kal/cmu_us_kal_residx.c",
    "lang/cmu_us_kal/cmu_us_kal_ressize.c",
    "lang/cmu_time_awb/cmu_time_awb.c",
    "lang/cmu_time_awb/cmu_time_awb_clunits.c",
    "lang/cmu_time_awb/cmu_time_awb_cart.c",
    "lang/cmu_time_awb/cmu_time_awb_mcep.c",
    "lang/cmu_time_awb/cmu_time_awb_lpc.c",
    "lang/cmu_time_awb/cmu_time_awb_lex_entry.c",
    "lang/cmu_us_kal16/cmu_us_kal16_diphone.c",
    "lang/cmu_us_kal16/cmu_us_kal16.c",
    "lang/cmu_us_kal16/cmu_us_kal16_lpc.c",
    "lang/cmu_us_kal16/cmu_us_kal16_res.c",
    "lang/cmu_us_kal16/cmu_us_kal16_residx.c",
    "lang/cmu_us_awb/cmu_us_awb.c",
    "lang/cmu_us_awb/cmu_us_awb_cg_single_mcep_trees.c",
    "lang/cmu_us_awb/cmu_us_awb_cg.c",
    "lang/cmu_us_awb/cmu_us_awb_cg_single_params.c",
    "lang/cmu_us_awb/cmu_us_awb_cg_durmodel.c",
    "lang/cmu_us_awb/cmu_us_awb_cg_phonestate.c",
    "lang/cmu_us_awb/cmu_us_awb_cg_f0_trees.c",
    "lang/cmu_us_awb/cmu_us_awb_spamf0_accent_params.c",
    "lang/cmu_us_awb/cmu_us_awb_spamf0_phrase.c",
    "lang/cmu_us_awb/cmu_us_awb_spamf0_accent.c",
    "lang/cmu_us_rms/cmu_us_rms.c",
    "lang/cmu_us_rms/cmu_us_rms_cg_single_mcep_trees.c",
    "lang/cmu_us_rms/cmu_us_rms_cg.c",
    "lang/cmu_us_rms/cmu_us_rms_cg_single_params.c",
    "lang/cmu_us_rms/cmu_us_rms_cg_durmodel.c",
    "lang/cmu_us_rms/cmu_us_rms_cg_phonestate.c",
    "lang/cmu_us_rms/cmu_us_rms_cg_f0_trees.c",
    "lang/cmu_us_rms/cmu_us_rms_spamf0_accent_params.c",
    "lang/cmu_us_rms/cmu_us_rms_spamf0_phrase.c",
    "lang/cmu_us_rms/cmu_us_rms_spamf0_accent.c",
    "lang/cmu_us_slt/cmu_us_slt.c",
    "lang/cmu_us_slt/cmu_us_slt_cg_single_mcep_trees.c",
    "lang/cmu_us_slt/cmu_us_slt_cg.c",
    "lang/cmu_us_slt/cmu_us_slt_cg_single_params.c",
    "lang/cmu_us_slt/cmu_us_slt_cg_durmodel.c",
    "lang/cmu_us_slt/cmu_us_slt_cg_phonestate.c",
    "lang/cmu_us_slt/cmu_us_slt_cg_f0_trees.c",
    "lang/cmu_us_slt/cmu_us_slt_spamf0_accent.c",
    "lang/cmu_us_slt/cmu_us_slt_spamf0_phrase.c",
    "lang/cmu_us_slt/cmu_us_slt_spamf0_accent_params.c",
];

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let vendor_path = manifest_dir.join("vendor");

    let mut builder = cc::Build::new();
    builder
        .files(
            FILE_LIST
                .iter()
                .map(|&p| vendor_path.join(p))
                .collect::<Vec<_>>(),
        )
        .file(manifest_dir.join("src/flite_voice_list.c"))
        .file(manifest_dir.join("src/flite_lang_list.c"))
        .include(vendor_path.join("include"))
        .include(vendor_path.join("lang/usenglish"))
        .include(vendor_path.join("lang/cmulex"))
        .opt_level(2)
        .define("CST_NO_SOCKETS", None)
        .define("CST_AUDIO_NONE", None)
        .warnings(false);

    if cfg!(target_os = "windows") {
        builder.define("UNDER_WINDOWS", None).define("WIN32", None);
    }

    builder.compile("flite");

    let bindings = bindgen::builder()
        .clang_arg(format!(
            "-I{}",
            vendor_path.join("include").to_string_lossy()
        ))
        .header(manifest_dir.join("bindings.h").to_string_lossy())
        .blocklist_item("_JUMP_BUFFER")
        .generate();

    let out_dir = env::var("OUT_DIR").unwrap();
    bindings
        .unwrap()
        .write_to_file(format!("{}/flite.rs", out_dir))
        .unwrap();
}
