use std::thread;

use ibc_test_framework::chain::ext::authz::AuthzMethodsExt;
use ibc_test_framework::prelude::*;

#[test]
fn test_authz() -> Result<(), Error> {
    run_binary_channel_test(&AuthzTest)
}

#[test]
fn test_no_authz() -> Result<(), Error> {
    run_binary_channel_test(&NoAuthzTest)
}

struct AuthzTest;

impl TestOverrides for AuthzTest {}

impl BinaryChannelTest for AuthzTest {
    fn run<ChainA: ChainHandle, ChainB: ChainHandle>(
        &self,
        _config: &TestConfig,
        _relayer: RelayerDriver,
        chains: ConnectedChains<ChainA, ChainB>,
        channels: ConnectedChannel<ChainA, ChainB>,
    ) -> Result<(), Error> {
        let denom_a = chains.node_a.denom();
        let wallet_b = chains.node_b.wallets().user1().cloned();

        let a_to_b_amount = 12345u64;
        let granter = chains
            .node_a
            .wallets()
            .user2()
            .address()
            .value()
            .to_string();
        let grantee = chains
            .node_a
            .wallets()
            .user1()
            .address()
            .value()
            .to_string();

        chains.node_a.chain_driver().authz_grant(
            &granter,
            &grantee,
            "/ibc.applications.transfer.v1.MsgTransfer",
        )?;

        chains.node_a.chain_driver().assert_eventual_grant(
            &granter,
            &grantee,
            "/ibc.applications.transfer.v1.MsgTransfer",
        )?;

        let granter_balance = chains
            .node_a
            .chain_driver()
            .query_balance(&chains.node_a.wallets().user2().address(), &denom_a)?;

        let denom_b = derive_ibc_denom(
            &channels.port_b.as_ref(),
            &channels.channel_id_b.as_ref(),
            &denom_a,
        )?;

        chains.node_a.chain_driver().exec_ibc_transfer_grant(
            &granter,
            &grantee,
            channels.port_a.value(),
            channels.channel_id_a.value(),
            &wallet_b.address(),
            &denom_a.with_amount(a_to_b_amount).as_ref(),
        )?;

        thread::sleep(Duration::from_secs(10));

        // Assert that user on chain B received the tokens
        chains.node_b.chain_driver().assert_eventual_wallet_amount(
            &wallet_b.address(),
            &denom_b.with_amount(a_to_b_amount).as_ref(),
        )?;

        // Assert that user on chain A sent the tokens
        chains.node_a.chain_driver().assert_eventual_wallet_amount(
            &chains.node_a.wallets().user2().address(),
            &(granter_balance - a_to_b_amount).as_ref(),
        )?;

        Ok(())
    }
}

struct NoAuthzTest;

impl TestOverrides for NoAuthzTest {}

impl BinaryChannelTest for NoAuthzTest {
    fn run<ChainA: ChainHandle, ChainB: ChainHandle>(
        &self,
        _config: &TestConfig,
        _relayer: RelayerDriver,
        chains: ConnectedChains<ChainA, ChainB>,
        channels: ConnectedChannel<ChainA, ChainB>,
    ) -> Result<(), Error> {
        let denom_a = chains.node_a.denom();
        let wallet_b = chains.node_b.wallets().user1().cloned();

        let a_to_b_amount = 12345u64;
        let granter = chains
            .node_a
            .wallets()
            .user2()
            .address()
            .value()
            .to_string();
        let grantee = chains
            .node_a
            .wallets()
            .user1()
            .address()
            .value()
            .to_string();

        let denom_b = derive_ibc_denom(
            &channels.port_b.as_ref(),
            &channels.channel_id_b.as_ref(),
            &denom_a,
        )?;

        assert!(
            chains
                .node_a
                .chain_driver()
                .assert_eventual_grant(
                    &granter,
                    &grantee,
                    "/ibc.applications.transfer.v1.MsgTransfer",
                )
                .is_err(),
            "there should be no grants"
        );

        let granter_balance = chains
            .node_a
            .chain_driver()
            .query_balance(&chains.node_a.wallets().user2().address(), &denom_a)?;

        assert!(
            chains
                .node_a
                .chain_driver()
                .exec_ibc_transfer_grant(
                    &granter,
                    &grantee,
                    channels.port_a.value(),
                    channels.channel_id_a.value(),
                    &wallet_b.address(),
                    &denom_a.with_amount(a_to_b_amount).as_ref(),
                )
                .is_err(),
            "expected authz grant exec to fail"
        );

        thread::sleep(Duration::from_secs(10));

        // Assert that user on chain B has not received tokens
        chains.node_b.chain_driver().assert_eventual_wallet_amount(
            &wallet_b.address(),
            &denom_b.with_amount(0u128).as_ref(),
        )?;

        // Assert that user on chain A has not sent tokens
        chains.node_a.chain_driver().assert_eventual_wallet_amount(
            &chains.node_a.wallets().user2().address(),
            &granter_balance.as_ref(),
        )?;

        Ok(())
    }
}
