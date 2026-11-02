//! Client device code contre un faux serveur HTTP (wiremock) : aucun accès réseau réel.

mod commun;

use std::time::Duration;

use tuxteams_application::ports::jetons::{DemandeCode, ErreurAuth, EtatSondage, PortConnexion};
use tuxteams_auth::{ClientDeviceCode, ConfigAuth, ErreurConfig};
use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const DEMANDE: &str = include_str!("fixtures/device_code/demande.json");
const ATTENTE: &str = include_str!("fixtures/device_code/attente.json");
const SUCCES: &str = include_str!("fixtures/device_code/succes.json");
const EXPIRE: &str = include_str!("fixtures/device_code/expire.json");
const COMPTE_PERSONNEL: &str = include_str!("fixtures/device_code/compte_personnel.json");

fn config(serveur: &MockServer) -> ConfigAuth {
    ConfigAuth::par_defaut().avec_autorite(Url::parse(&serveur.uri()).expect("url du serveur"))
}

fn client(config: ConfigAuth) -> ClientDeviceCode {
    ClientDeviceCode::new(config).expect("client créable")
}

async fn repondre(serveur: &MockServer, chemin: &str, statut: u16, corps: &str) {
    Mock::given(method("POST"))
        .and(path(chemin))
        .respond_with(
            ResponseTemplate::new(statut).set_body_raw(corps.to_owned(), "application/json"),
        )
        .mount(serveur)
        .await;
}

/// Champs du corps du dernier appel, décodés comme un formulaire.
async fn formulaire(serveur: &MockServer) -> Vec<(String, String)> {
    let requetes = serveur
        .received_requests()
        .await
        .expect("requêtes enregistrées");
    let derniere = requetes.last().expect("au moins une requête");
    url::form_urlencoded::parse(&derniere.body)
        .map(|(cle, valeur)| (cle.into_owned(), valeur.into_owned()))
        .collect()
}

fn champ(formulaire: &[(String, String)], nom: &str) -> Option<String> {
    formulaire
        .iter()
        .find(|(cle, _)| cle == nom)
        .map(|(_, v)| v.clone())
}

fn demande_fictive() -> DemandeCode {
    DemandeCode {
        code_utilisateur: "ABCD1234".into(),
        code_appareil: "DEVICE-CODE-FICTIF".into(),
        url_verification: Url::parse("https://login.microsoft.com/device").expect("url"),
        expire_dans: Duration::from_secs(900),
        intervalle: Duration::from_secs(5),
    }
}

const CHEMIN_DEMANDE: &str = "/organizations/oauth2/v2.0/devicecode";
const CHEMIN_JETON: &str = "/organizations/oauth2/v2.0/token";

#[tokio::test]
async fn la_demande_de_code_envoie_le_client_id_et_les_scopes_attendus() {
    let serveur = MockServer::start().await;
    repondre(&serveur, CHEMIN_DEMANDE, 200, DEMANDE).await;

    let demande = client(config(&serveur))
        .start_device_code()
        .await
        .expect("code émis");

    let corps = formulaire(&serveur).await;
    assert_eq!(
        champ(&corps, "client_id").as_deref(),
        Some("1fec8e78-bce4-4aaf-ab1b-5451cc387264")
    );
    assert_eq!(
        champ(&corps, "scope").as_deref(),
        Some("https://api.spaces.skype.com/.default openid profile offline_access")
    );
    assert_eq!(demande.code_utilisateur, "ABCD1234");
    assert_eq!(
        demande.url_verification.as_str(),
        "https://login.microsoft.com/device"
    );
    assert_eq!(demande.expire_dans, Duration::from_secs(900));
    assert_eq!(demande.intervalle, Duration::from_secs(5));
}

#[tokio::test]
async fn le_client_id_est_surchargeable_par_configuration() {
    let serveur = MockServer::start().await;
    repondre(&serveur, CHEMIN_DEMANDE, 200, DEMANDE).await;
    let config = ConfigAuth::depuis_variables(|nom| {
        (nom == "TUXTEAMS_CLIENT_ID").then(|| "11111111-2222-3333-4444-555555555555".to_owned())
    })
    .expect("configuration valide")
    .avec_autorite(Url::parse(&serveur.uri()).expect("url"));

    client(config).start_device_code().await.expect("code émis");

    let corps = formulaire(&serveur).await;
    assert_eq!(
        champ(&corps, "client_id").as_deref(),
        Some("11111111-2222-3333-4444-555555555555")
    );
}

#[tokio::test]
async fn l_autorite_utilise_le_tenant_configure() {
    let serveur = MockServer::start().await;
    repondre(
        &serveur,
        "/contoso.onmicrosoft.com/oauth2/v2.0/devicecode",
        200,
        DEMANDE,
    )
    .await;
    let config = ConfigAuth::depuis_variables(|nom| {
        (nom == "TUXTEAMS_TENANT").then(|| "contoso.onmicrosoft.com".to_owned())
    })
    .expect("configuration valide")
    .avec_autorite(Url::parse(&serveur.uri()).expect("url"));

    client(config)
        .start_device_code()
        .await
        .expect("le chemin du tenant configuré est appelé");
}

#[test]
fn un_tenant_avec_des_caracteres_de_chemin_est_refuse() {
    let resultat = ConfigAuth::depuis_variables(|nom| {
        (nom == "TUXTEAMS_TENANT").then(|| "../evil".to_owned())
    });
    assert_eq!(
        resultat,
        Err(ErreurConfig::ValeurInvalide("TUXTEAMS_TENANT"))
    );
}

#[tokio::test]
async fn une_erreur_oauth_inconnue_est_typee_sans_panic() {
    let serveur = MockServer::start().await;
    repondre(
        &serveur,
        CHEMIN_DEMANDE,
        400,
        r#"{"error":"invalid_client","error_description":"AADSTS700016: détail sensible"}"#,
    )
    .await;

    let erreur = client(config(&serveur))
        .start_device_code()
        .await
        .expect_err("erreur typée");

    assert_eq!(erreur, ErreurAuth::Inattendu("invalid_client".into()));
    assert!(
        !erreur.to_string().contains("sensible"),
        "la description ne doit pas fuiter"
    );
}

#[tokio::test]
async fn un_corps_non_json_ou_incomplet_est_une_erreur_typee() {
    let serveur = MockServer::start().await;
    repondre(&serveur, CHEMIN_DEMANDE, 502, "<html>Bad gateway</html>").await;
    let erreur = client(config(&serveur))
        .start_device_code()
        .await
        .expect_err("erreur typée");
    assert_eq!(erreur, ErreurAuth::Inattendu("corps_illisible".into()));

    let serveur = MockServer::start().await;
    repondre(&serveur, CHEMIN_DEMANDE, 200, r#"{"interval":5}"#).await;
    let erreur = client(config(&serveur))
        .start_device_code()
        .await
        .expect_err("erreur typée");
    assert!(matches!(erreur, ErreurAuth::Inattendu(_)), "{erreur:?}");
}

#[tokio::test]
async fn une_url_de_verification_non_https_est_refusee() {
    let serveur = MockServer::start().await;
    let corps = DEMANDE.replace(
        "https://login.microsoft.com/device",
        "http://exemple.invalide/device",
    );
    repondre(&serveur, CHEMIN_DEMANDE, 200, &corps).await;

    let erreur = client(config(&serveur))
        .start_device_code()
        .await
        .expect_err("refusée");

    assert!(matches!(erreur, ErreurAuth::Inattendu(_)), "{erreur:?}");
}

#[tokio::test]
async fn un_intervalle_nul_vaut_cinq_secondes() {
    let serveur = MockServer::start().await;
    repondre(
        &serveur,
        CHEMIN_DEMANDE,
        200,
        &DEMANDE.replace("\"interval\": 5", "\"interval\": 0"),
    )
    .await;

    let demande = client(config(&serveur))
        .start_device_code()
        .await
        .expect("code émis");

    assert_eq!(demande.intervalle, Duration::from_secs(5));
}

#[tokio::test]
async fn un_serveur_injoignable_donne_une_erreur_reseau() {
    // Port local réservé puis libéré : plus rien n'y écoute.
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .expect("port libre")
        .local_addr()
        .expect("adresse")
        .port();
    let config = ConfigAuth::par_defaut()
        .avec_autorite(Url::parse(&format!("http://127.0.0.1:{port}")).expect("url"));

    let erreur = client(config)
        .start_device_code()
        .await
        .expect_err("injoignable");

    assert_eq!(erreur, ErreurAuth::Reseau);
}

#[tokio::test]
async fn le_sondage_envoie_le_code_d_appareil_et_le_type_de_grant() {
    let serveur = MockServer::start().await;
    repondre(&serveur, CHEMIN_JETON, 400, ATTENTE).await;

    client(config(&serveur))
        .poll_device_code(&demande_fictive())
        .await
        .expect("attente");

    let corps = formulaire(&serveur).await;
    assert_eq!(
        champ(&corps, "grant_type").as_deref(),
        Some("urn:ietf:params:oauth:grant-type:device_code")
    );
    assert_eq!(
        champ(&corps, "device_code").as_deref(),
        Some("DEVICE-CODE-FICTIF")
    );
    assert_eq!(
        champ(&corps, "client_id").as_deref(),
        Some("1fec8e78-bce4-4aaf-ab1b-5451cc387264")
    );
}

#[tokio::test]
async fn authorization_pending_donne_en_attente() {
    let serveur = MockServer::start().await;
    repondre(&serveur, CHEMIN_JETON, 400, ATTENTE).await;
    let etat = client(config(&serveur))
        .poll_device_code(&demande_fictive())
        .await;
    assert_eq!(etat, Ok(EtatSondage::EnAttente));
}

#[tokio::test]
async fn slow_down_donne_plus_lent() {
    let serveur = MockServer::start().await;
    repondre(&serveur, CHEMIN_JETON, 400, r#"{"error":"slow_down"}"#).await;
    let etat = client(config(&serveur))
        .poll_device_code(&demande_fictive())
        .await;
    assert_eq!(etat, Ok(EtatSondage::PlusLent));
}

#[tokio::test]
async fn un_jeton_valide_donne_le_compte_de_l_id_token() {
    let serveur = MockServer::start().await;
    repondre(&serveur, CHEMIN_JETON, 200, &commun::reponse_jeton(SUCCES)).await;

    let etat = client(config(&serveur))
        .poll_device_code(&demande_fictive())
        .await;

    let Ok(EtatSondage::Termine(authentifie)) = etat else {
        panic!("état inattendu : {etat:?}")
    };
    let compte = authentifie.compte;
    assert_eq!(compte.nom.as_deref(), Some("Alex Example"));
    assert_eq!(compte.email.as_deref(), Some("alex@contoso.example"));
    assert_eq!(compte.tenant, "00000000-0000-0000-0000-0000000000aa");
}

#[tokio::test]
async fn la_connexion_remet_le_refresh_token_recu_sans_l_exposer_par_debug() {
    let serveur = MockServer::start().await;
    repondre(&serveur, CHEMIN_JETON, 200, &commun::reponse_jeton(SUCCES)).await;

    let etat = client(config(&serveur))
        .poll_device_code(&demande_fictive())
        .await;

    let Ok(EtatSondage::Termine(authentifie)) = etat else {
        panic!("état inattendu : {etat:?}")
    };
    let refresh = authentifie.refresh.as_ref().expect("refresh token remis");
    assert_eq!(refresh.exposer(), commun::VALEUR_RENOUVELLEMENT_FICTIVE);
    let sortie = format!("{authentifie:?}");
    assert!(
        !sortie.contains(commun::VALEUR_RENOUVELLEMENT_FICTIVE),
        "{sortie}"
    );
}

#[tokio::test]
async fn un_jeton_de_compte_personnel_est_refuse() {
    let serveur = MockServer::start().await;
    repondre(
        &serveur,
        CHEMIN_JETON,
        200,
        &commun::reponse_jeton(COMPTE_PERSONNEL),
    )
    .await;
    let etat = client(config(&serveur))
        .poll_device_code(&demande_fictive())
        .await;
    assert_eq!(etat, Err(ErreurAuth::ComptePersonnel));
}

#[tokio::test]
async fn un_code_expire_est_type() {
    let serveur = MockServer::start().await;
    repondre(&serveur, CHEMIN_JETON, 400, EXPIRE).await;
    let etat = client(config(&serveur))
        .poll_device_code(&demande_fictive())
        .await;
    assert_eq!(etat, Err(ErreurAuth::CodeExpire));
}

#[tokio::test]
async fn un_refus_de_l_utilisateur_est_type() {
    let serveur = MockServer::start().await;
    repondre(
        &serveur,
        CHEMIN_JETON,
        400,
        r#"{"error":"authorization_declined"}"#,
    )
    .await;
    let etat = client(config(&serveur))
        .poll_device_code(&demande_fictive())
        .await;
    assert_eq!(etat, Err(ErreurAuth::Refuse));
}

#[tokio::test]
async fn l_acces_conditionnel_aadsts53003_est_type() {
    let serveur = MockServer::start().await;
    repondre(
        &serveur,
        CHEMIN_JETON,
        400,
        r#"{"error":"interaction_required","error_description":"AADSTS53003: Access has been blocked by Conditional Access policies."}"#,
    )
    .await;
    let etat = client(config(&serveur))
        .poll_device_code(&demande_fictive())
        .await;
    assert_eq!(etat, Err(ErreurAuth::AccesConditionnel));
}

#[tokio::test]
async fn un_jeton_sans_id_token_est_une_erreur_typee() {
    let serveur = MockServer::start().await;
    repondre(
        &serveur,
        CHEMIN_JETON,
        200,
        &commun::reponse_sans_id_token(),
    )
    .await;
    let etat = client(config(&serveur))
        .poll_device_code(&demande_fictive())
        .await;
    assert!(matches!(etat, Err(ErreurAuth::Inattendu(_))), "{etat:?}");
}

#[tokio::test]
async fn une_reponse_de_jeton_de_20_kio_est_acceptee() {
    let serveur = MockServer::start().await;
    let corps = commun::reponse_jeton_volumineuse(SUCCES, 20 * 1024);
    repondre(&serveur, CHEMIN_JETON, 200, &corps).await;

    let etat = client(config(&serveur))
        .poll_device_code(&demande_fictive())
        .await;

    assert!(matches!(etat, Ok(EtatSondage::Termine(_))), "{etat:?}");
}

#[tokio::test]
async fn une_reponse_au_dela_de_la_limite_est_une_erreur_typee() {
    let serveur = MockServer::start().await;
    let corps = commun::reponse_jeton_volumineuse(SUCCES, 100 * 1024);
    repondre(&serveur, CHEMIN_JETON, 200, &corps).await;

    let etat = client(config(&serveur))
        .poll_device_code(&demande_fictive())
        .await;

    assert_eq!(
        etat,
        Err(ErreurAuth::Inattendu("reponse_trop_grande".into()))
    );
}
