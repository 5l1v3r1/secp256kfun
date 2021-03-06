#![allow(non_upper_case_globals)]
use criterion::{criterion_group, criterion_main, Criterion};
use schnorr_fun::Schnorr;
use secp256kfun::{hash::Derivation, marker::*, Scalar};

const MESSAGE: &'static [u8; 32] = b"hello world you are beautiful!!!";

lazy_static::lazy_static! {
    static ref SK: Scalar<Secret,NonZero> = Scalar::from_bytes_mod_order(*b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").mark::<NonZero>().unwrap();
    static ref schnorr: Schnorr = schnorr_fun::Schnorr::from_tag(b"bench");
}

// note schnorr runs against grin's secp256k1 library
fn sign_schnorr(c: &mut Criterion) {
    let mut group = c.benchmark_group("schnorr_sign");
    let keypair = schnorr.new_keypair(SK.clone());
    {
        group.bench_function("fun::schnorr_sign", |b| {
            b.iter(|| schnorr.sign(&keypair, &MESSAGE[..], Derivation::Deterministic))
        });
    }

    {
        use secp256k1zkp::{
            aggsig,
            key::{PublicKey, SecretKey},
            Message, Secp256k1,
        };

        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&secp, SK.to_bytes().as_ref()).unwrap();
        let pk = PublicKey::from_secret_key(&secp, &secret_key).unwrap();
        let msg = Message::from_slice(&MESSAGE[..]).unwrap();
        group.bench_function("grin::aggsig::sign_single", |b| {
            b.iter(|| {
                aggsig::sign_single(&secp, &msg, &secret_key, None, None, None, Some(&pk), None)
            });
        });
    }
}

fn verify_schnorr(c: &mut Criterion) {
    let mut group = c.benchmark_group("schnorr_verify");
    let keypair = schnorr.new_keypair(SK.clone());
    {
        let sig = schnorr.sign(&keypair, &MESSAGE[..], Derivation::Deterministic);
        let verification_key = &keypair.verification_key();
        group.bench_function("fun::schnorr_verify", |b| {
            b.iter(|| schnorr.verify(&verification_key, &MESSAGE[..], &sig))
        });

        {
            let sig = sig.clone().mark::<Secret>();
            group.bench_function("fun::schnorr_verify_ct", |b| {
                b.iter(|| schnorr.verify(&verification_key, &MESSAGE[..], &sig))
            });
        }
    }

    {
        use secp256k1zkp::{
            aggsig,
            key::{PublicKey, SecretKey},
            Message, Secp256k1,
        };
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&secp, SK.to_bytes().as_ref()).unwrap();
        let pk = PublicKey::from_secret_key(&secp, &secret_key).unwrap();
        let msg = Message::from_slice(&MESSAGE[..]).unwrap();

        let sig = aggsig::sign_single(&secp, &msg, &secret_key, None, None, None, Some(&pk), None) //
            .unwrap();
        assert!(aggsig::verify_single(
            &secp,
            &sig,
            &msg,
            None,
            &pk,
            Some(&pk),
            None,
            false
        ));
        group.bench_function("grin::aggsig::verify_single", |b| {
            b.iter(|| aggsig::verify_single(&secp, &sig, &msg, None, &pk, Some(&pk), None, false))
        });
    }
}

criterion_group!(benches, verify_schnorr, sign_schnorr);
criterion_main!(benches);
