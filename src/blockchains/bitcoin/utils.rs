use std::str::FromStr;

use bitcoin::{
    hashes::Hash,
    key::{rand::rngs::OsRng, Secp256k1},
    secp256k1::{SecretKey, Signing},
    Address, Amount, CompressedPublicKey, KnownHrp, Network, NetworkKind, OutPoint, PrivateKey,
    ScriptBuf, TxOut, Txid, WPubkeyHash,
};
use secp256k1::rand;

pub const DUMMY_UTXO_AMOUNT: Amount = Amount::from_sat(6600);
pub const SPEND_AMOUNT: Amount = Amount::from_sat(1000);
pub const CHANGE_AMOUNT: Amount = Amount::from_sat(5400); // 100 sat fee.

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

/// An example of keys controlled by the transaction sender.
///
/// In a real application these would be actual secrets.
pub fn senders_keys<C: Signing>(secp: &Secp256k1<C>) -> (SecretKey, WPubkeyHash) {
    let private_key = PrivateKey::from_wif("cSjgVro2xkCVat8fjye1jNfozoaC8XASd3UuvLXF49ugaZx1MHsg")
        .expect("invalid wif");
    let sk = private_key.inner;
    let pk = bitcoin::PublicKey::new(sk.public_key(secp));
    let wpkh = pk.wpubkey_hash().expect("key is compressed");

    (sk, wpkh)
}

/// A dummy address for the receiver.
///
/// We lock the spend output to the key associated with this address.
///
/// (FWIW this is a random mainnet address from block 80219.)
pub fn receivers_address() -> Address {
    "tb1qpuv7hp0khx0u65k2868nksx6gxynfgn88rd2gs"
        .parse::<Address<_>>()
        .expect("a valid address")
        .require_network(Network::Testnet)
        .expect("valid address for testnet")
}

/// Constructs a new p2wpkh output locked to the key associated with `wpkh`.
///
/// An utxo is described by the `OutPoint` (txid and index within the transaction that it was
/// created). Using the out point one can get the transaction by `txid` and using the `vout` get the
/// transaction value and script pubkey (`TxOut`) of the utxo.
///
/// This output is locked to keys that we control, in a real application this would be a valid
/// output taken from a transaction that appears in the chain.
pub fn dummy_unspent_transaction_output(wpkh: WPubkeyHash) -> (OutPoint, TxOut) {
    let script_pubkey = ScriptBuf::new_p2wpkh(&wpkh);

    let out_point = OutPoint {
        txid: Txid::from_str("f924b4293fd009ed51f4ec69c60ae16b6d6d79616d1b76cc01a443c969d69f00")
            .unwrap(), // Arbitrary invalid dummy value.
        vout: 1,
    };

    let utxo = TxOut {
        value: DUMMY_UTXO_AMOUNT,
        script_pubkey,
    };

    (out_point, utxo)
}
