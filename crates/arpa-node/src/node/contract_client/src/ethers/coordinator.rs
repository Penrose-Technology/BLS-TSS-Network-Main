use crate::{
    contract_stub::coordinator::Coordinator,
    coordinator::{
        CoordinatorClientBuilder, CoordinatorTransactions, CoordinatorViews, DKGContractError,
    },
    error::ContractClientResult,
    ServiceClient, TransactionCaller, ViewCaller,
};
use arpa_node_core::{
    ChainIdentity, ExponentialBackoffRetryDescriptor, GeneralChainIdentity, WalletSigner,
};
use async_trait::async_trait;
use dkg_core::{
    primitives::{BundledJustification, BundledResponses, BundledShares},
    BoardPublisher,
};
use ethers::prelude::*;
use log::info;
use std::sync::Arc;
use threshold_bls::group::Curve;

pub struct CoordinatorClient {
    coordinator_address: Address,
    signer: Arc<WalletSigner>,
    contract_transaction_retry_descriptor: ExponentialBackoffRetryDescriptor,
    contract_view_retry_descriptor: ExponentialBackoffRetryDescriptor,
}

impl CoordinatorClient {
    pub fn new(
        coordinator_address: Address,
        identity: &GeneralChainIdentity,
        contract_transaction_retry_descriptor: ExponentialBackoffRetryDescriptor,
        contract_view_retry_descriptor: ExponentialBackoffRetryDescriptor,
    ) -> Self {
        CoordinatorClient {
            coordinator_address,
            signer: identity.get_signer(),
            contract_transaction_retry_descriptor,
            contract_view_retry_descriptor,
        }
    }
}

impl<C: Curve + 'static> CoordinatorClientBuilder<C> for GeneralChainIdentity {
    type Service = CoordinatorClient;

    fn build_coordinator_client(&self, contract_address: Address) -> CoordinatorClient {
        CoordinatorClient::new(
            contract_address,
            self,
            self.get_contract_transaction_retry_descriptor(),
            self.get_contract_view_retry_descriptor(),
        )
    }
}

type CoordinatorContract = Coordinator<WalletSigner>;

#[async_trait]
impl ServiceClient<CoordinatorContract> for CoordinatorClient {
    async fn prepare_service_client(&self) -> ContractClientResult<CoordinatorContract> {
        let coordinator_contract = Coordinator::new(self.coordinator_address, self.signer.clone());

        Ok(coordinator_contract)
    }
}

#[async_trait]
impl TransactionCaller for CoordinatorClient {}

#[async_trait]
impl ViewCaller for CoordinatorClient {}

#[async_trait]
impl CoordinatorTransactions for CoordinatorClient {
    async fn publish(&self, value: Vec<u8>) -> ContractClientResult<H256> {
        let coordinator_contract =
            ServiceClient::<CoordinatorContract>::prepare_service_client(self).await?;

        let call = coordinator_contract.publish(value.into());

        CoordinatorClient::call_contract_transaction(
            "publish",
            call,
            self.contract_transaction_retry_descriptor,
            false,
        )
        .await
    }
}

#[async_trait]
impl CoordinatorViews for CoordinatorClient {
    async fn get_shares(&self) -> ContractClientResult<Vec<Vec<u8>>> {
        let coordinator_contract =
            ServiceClient::<CoordinatorContract>::prepare_service_client(self).await?;

        CoordinatorClient::call_contract_view(
            "get_shares",
            coordinator_contract.get_shares(),
            self.contract_view_retry_descriptor,
        )
        .await
        .map(|r| r.iter().map(|b| b.to_vec()).collect::<Vec<Vec<u8>>>())
    }

    async fn get_responses(&self) -> ContractClientResult<Vec<Vec<u8>>> {
        let coordinator_contract =
            ServiceClient::<CoordinatorContract>::prepare_service_client(self).await?;

        CoordinatorClient::call_contract_view(
            "get_responses",
            coordinator_contract.get_responses(),
            self.contract_view_retry_descriptor,
        )
        .await
        .map(|r| r.iter().map(|b| b.to_vec()).collect::<Vec<Vec<u8>>>())
    }

    async fn get_justifications(&self) -> ContractClientResult<Vec<Vec<u8>>> {
        let coordinator_contract =
            ServiceClient::<CoordinatorContract>::prepare_service_client(self).await?;

        CoordinatorClient::call_contract_view(
            "get_justifications",
            coordinator_contract.get_justifications(),
            self.contract_view_retry_descriptor,
        )
        .await
        .map(|r| r.iter().map(|b| b.to_vec()).collect::<Vec<Vec<u8>>>())
    }

    async fn get_participants(&self) -> ContractClientResult<Vec<Address>> {
        let coordinator_contract =
            ServiceClient::<CoordinatorContract>::prepare_service_client(self).await?;

        CoordinatorClient::call_contract_view(
            "get_participants",
            coordinator_contract.get_participants(),
            self.contract_view_retry_descriptor,
        )
        .await
    }

    async fn get_dkg_keys(&self) -> ContractClientResult<(usize, Vec<Vec<u8>>)> {
        let coordinator_contract =
            ServiceClient::<CoordinatorContract>::prepare_service_client(self).await?;

        CoordinatorClient::call_contract_view(
            "get_dkg_keys",
            coordinator_contract.get_dkg_keys(),
            self.contract_view_retry_descriptor,
        )
        .await
        .map(|(t, keys)| {
            (
                t.as_usize(),
                keys.iter().map(|b| b.to_vec()).collect::<Vec<Vec<u8>>>(),
            )
        })
    }

    async fn in_phase(&self) -> ContractClientResult<i8> {
        let coordinator_contract =
            ServiceClient::<CoordinatorContract>::prepare_service_client(self).await?;

        CoordinatorClient::call_contract_view(
            "in_phase",
            coordinator_contract.in_phase(),
            self.contract_view_retry_descriptor,
        )
        .await
    }
}

#[async_trait]
impl<C: Curve + 'static> BoardPublisher<C> for CoordinatorClient {
    type Error = DKGContractError;

    async fn publish_shares(&mut self, shares: BundledShares<C>) -> Result<(), Self::Error> {
        info!("called publish_shares");
        let serialized = bincode::serialize(&shares)?;
        self.publish(serialized).await?;
        Ok(())
    }

    async fn publish_responses(&mut self, responses: BundledResponses) -> Result<(), Self::Error> {
        info!("called publish_responses");
        let serialized = bincode::serialize(&responses)?;
        self.publish(serialized).await?;
        Ok(())
    }

    async fn publish_justifications(
        &mut self,
        justifications: BundledJustification<C>,
    ) -> Result<(), Self::Error> {
        let serialized = bincode::serialize(&justifications)?;
        self.publish(serialized).await?;
        Ok(())
    }
}

#[cfg(test)]
pub mod coordinator_tests {
    use super::{CoordinatorClient, WalletSigner};
    use crate::contract_stub::coordinator::Coordinator;
    use crate::coordinator::CoordinatorTransactions;
    use arpa_node_core::Config;
    use arpa_node_core::GeneralChainIdentity;
    use ethers::abi::Tokenize;
    use ethers::prelude::*;
    use ethers::signers::coins_bip39::English;
    use ethers::utils::Anvil;
    use ethers::utils::AnvilInstance;
    use std::env;
    use std::path::PathBuf;
    use std::{convert::TryFrom, sync::Arc, time::Duration};
    use threshold_bls::schemes::bn254::G2Scheme;

    #[test]
    fn test_cargo_manifest_parent_dir() {
        let dir = env!("CARGO_MANIFEST_DIR");
        println!("{:?}", PathBuf::new().join(dir).parent());
        println!("{:?}", (3u8, 10u8).into_tokens());
    }

    const PHRASE: &str =
        "work man father plunge mystery proud hollow address reunion sauce theory bonus";
    const INDEX: u32 = 0;

    fn start_chain() -> AnvilInstance {
        Anvil::new().chain_id(1u64).mnemonic(PHRASE).spawn()
    }

    async fn deploy_contract(anvil: &AnvilInstance) -> Coordinator<WalletSigner> {
        // 2. instantiate our wallet
        let wallet: LocalWallet = anvil.keys()[0].clone().into();

        // 3. connect to the network
        let provider = Provider::<Http>::try_from(anvil.endpoint())
            .unwrap()
            .interval(Duration::from_millis(3000));

        // 4. instantiate the client with the wallet
        let nonce_manager = NonceManagerMiddleware::new(Arc::new(provider), wallet.address());

        let client = Arc::new(SignerMiddleware::new(
            nonce_manager,
            wallet.with_chain_id(anvil.chain_id()),
        ));

        // 5. deploy contract
        let coordinator_contract = Coordinator::deploy(client, (3u8, 30u8))
            .unwrap()
            .send()
            .await
            .unwrap();

        coordinator_contract
    }

    #[tokio::test]
    async fn test_coordinator_in_phase() {
        let anvil = start_chain();
        let coordinator_contract = deploy_contract(&anvil).await;
        let res = coordinator_contract.in_phase().call().await.unwrap();

        println!("{:?}", res);
    }

    #[tokio::test]
    async fn test_publish_to_coordinator() {
        let config = Config::default().initialize();

        let anvil = start_chain();
        let coordinator_contract = deploy_contract(&anvil).await;

        let wallet = MnemonicBuilder::<English>::default()
            .phrase(PHRASE)
            .index(INDEX)
            .unwrap()
            .build()
            .unwrap();

        // mock dkg key pair
        let (_, dkg_public_key) = dkg_core::generate_keypair::<G2Scheme>();

        let nodes = vec![wallet.address()];
        let public_keys = vec![bincode::serialize(&dkg_public_key).unwrap().into()];

        coordinator_contract
            .initialize(nodes, public_keys)
            .send()
            .await
            .unwrap();

        let main_chain_identity = GeneralChainIdentity::new(
            anvil.chain_id() as usize,
            wallet,
            anvil.endpoint(),
            3000,
            Address::random(),
            Address::random(),
            config
                .time_limits
                .unwrap()
                .contract_transaction_retry_descriptor,
            config.time_limits.unwrap().contract_view_retry_descriptor,
        );

        let client = CoordinatorClient::new(
            coordinator_contract.address(),
            &main_chain_identity,
            config
                .time_limits
                .unwrap()
                .contract_transaction_retry_descriptor,
            config.time_limits.unwrap().contract_view_retry_descriptor,
        );

        let mock_value = vec![1, 2, 3, 4];
        let res = client.publish(mock_value.clone()).await;
        assert!(res.is_ok());

        let res = client.publish(mock_value.clone()).await;
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.to_string().contains("share existed"));
    }

    #[test]
    fn test_three_ways_to_provide_wallet() {
        //1. mnemonic

        // Access mnemonic phrase with password
        // Child key at derivation path: m/44'/60'/0'/0/{index}
        let password = "TREZOR123";

        let wallet1 = MnemonicBuilder::<English>::default()
            .phrase(PHRASE)
            .index(INDEX)
            .unwrap()
            // Use this if your mnemonic is encrypted
            .password(password)
            .build()
            .unwrap();

        // 2.private key in plaintext
        let wallet2: LocalWallet =
            "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
                .parse()
                .unwrap();

        // 3. private key in keystore(protected by password)
        let path = PathBuf::new().join(env!("CARGO_MANIFEST_DIR"));
        let mut rng = rand::thread_rng();
        let (_key, _uuid) =
            LocalWallet::new_keystore(&path, &mut rng, "randpsswd", Some("passwd")).unwrap();

        // read from the encrypted JSON keystore and decrypt it, while validating that the
        // signatures produced by both the keys should match

        let wallet3 = LocalWallet::decrypt_keystore(&path.join("passwd"), "randpsswd").unwrap();
        // let signature2 = key2.sign_message(message).await.unwrap();

        println!("{:?}", wallet1);
        println!("{:?}", wallet2);
        println!("{:?}", wallet3);
    }
}
