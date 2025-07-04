use bitcoin::{
    key::{rand::rngs::OsRng, Secp256k1},
    secp256k1::{SecretKey, Signing},
    Address, CompressedPublicKey, KnownHrp, Network, NetworkKind, PrivateKey, WPubkeyHash,
};
use std::str::FromStr;
use tracing::info;

// This function is used to create a testnet wallet
// Warning : Actual keys of a bitcoin wallet.
#[allow(dead_code)]
pub fn create_testnet_wallet() -> (PrivateKey, Address) {
    //1.Create a random private key
    let secp = Secp256k1::new();
    let mut rng = OsRng;
    let secret_key = SecretKey::new(&mut rng);

    //2. Convert it into a Bitcoin PrivateKey (Testnet)
    let private_key = PrivateKey {
        compressed: true,
        network: NetworkKind::Test,
        inner: secret_key,
    };

    //3️. Generate a compressed Public Key
    let public_key = CompressedPublicKey::from_private_key(&secp, &private_key).unwrap();

    //4. Generate a Testnet Bech32 Address (P2WPKH)
    let address = Address::p2wpkh(&public_key, KnownHrp::Testnets);

    // 🔹 Print the wallet details
    println!("🔑 Private Key (WIF): {}", private_key.to_wif());
    println!("📫 Testnet Address: {}", address);

    (private_key, address)
}

// This function is used to get the secret key and the witness public key hash of the sender.
// Warning : Actual keys of a bitcoin wallet.
pub fn senders_keys<C: Signing>(secp: &Secp256k1<C>) -> (SecretKey, WPubkeyHash) {
    let private_key = PrivateKey::from_wif("cSjgVro2xkCVat8fjye1jNfozoaC8XASd3UuvLXF49ugaZx1MHsg")
        .expect("invalid wif");
    let sk = private_key.inner;
    let pk = bitcoin::PublicKey::new(sk.public_key(secp));
    let wpkh = pk.wpubkey_hash().expect("key is compressed");

    (sk, wpkh)
}

pub fn is_valid_bitcoin_address(address: &str, network: Network) -> bool {
    match Address::from_str(address) {
        Ok(addr) => match addr.require_network(network) {
            Ok(_validated_address) => true,
            Err(err) => {
                info!("Invalid Bitcoin address {} : {}", address, err);
                false
            }
        },
        Err(err) => {
            info!("Unable to parse Bitcoin address {} : {}", address, err);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_bitcoin_address() {
        let address = "tb1q04sf26d069dsfmd079n3ehjzevd9nzewmzg9mf4xk";
        let is_valid = is_valid_bitcoin_address(address, Network::Testnet);
        println!("{}", is_valid);
        assert!(!is_valid);
    }
}
