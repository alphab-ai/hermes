use async_trait::async_trait;
use ibc::core::ics04_channel::packet::Sequence;
use ibc::core::ics24_host::identifier::{ChannelId, PortId};
use ibc::Height;

use super::context::RelayContext;
use crate::tagged::{DualTagged, MonoTagged};
use crate::types::message::Message;

#[async_trait]
pub trait AckPacketMessageBuilder: RelayContext {
    async fn build_ack_packet_message(
        &self,
        height: MonoTagged<Self::DstChain, Height>,
        port_id: DualTagged<Self::DstChain, Self::SrcChain, PortId>,
        channel_id: DualTagged<Self::DstChain, Self::SrcChain, ChannelId>,
        sequence: DualTagged<Self::SrcChain, Self::DstChain, Sequence>,
    ) -> Result<Message<Self::SrcChain, Self::DstChain>, Self::Error>;
}
