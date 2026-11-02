fn main() {
    // Les informations de débogage permettent au backend de test de retrouver les éléments.
    let debug = std::env::var("PROFILE").is_ok_and(|profil| profil == "debug");
    let configuration = slint_build::CompilerConfiguration::new().with_debug_info(debug);
    slint_build::compile_with_config("ui/fenetre.slint", configuration)
        .expect("compilation Slint impossible");
}
