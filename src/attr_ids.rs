use crate::error::Error;

/// Attribute ID.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum AttributeId {
    /// Message level attribute IDs.
    Message(MessageAttrId),
    /// Attachment level attribute IDs.
    Attachment(AttachAttrId),
}

impl AttributeId {
    pub(crate) fn from_u32(is_msg: bool, id: u32) -> Result<Self, Error> {
        Ok(match (is_msg, id) {
            (true, id) => Self::Message(MessageAttrId::from_u32(id)?),
            (false, id) => Self::Attachment(AttachAttrId::from_u32(id)?),
        })
    }
}

/// Message level attribute IDs.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MessageAttrId {
    MessageClass,
    From,
    Subject,
    DateSent,
    DateRecd,
    MessageStatus,
    MessageID,
    ConversationID,
    Body,
    Priority,
    DateModified,
    MsgProps,
    RecipTable,
    OriginalMessageClass,
    Owner,
    SentFor,
    Delegate,
    DateStart,
    DateEnd,
    AidOwner,
    RequestRes,
}

impl MessageAttrId {
    pub(crate) fn from_u32(id: u32) -> Result<Self, Error> {
        use MessageAttrId::*;
        Ok(match id {
            0x0007_8008 => MessageClass,
            0x0000_8000 => From,
            0x0001_8004 => Subject,
            0x0003_8005 => DateSent,
            0x0003_8006 => DateRecd,
            0x0006_8007 => MessageStatus,
            0x0001_8009 => MessageID,
            0x0001_800B => ConversationID,
            0x0004_8020 => Body,
            0x0004_800D => Priority,
            0x0003_8020 => DateModified,
            0x0006_9003 => MsgProps,
            0x0006_9004 => RecipTable,
            0x0007_0600 => OriginalMessageClass,
            0x0006_0000 => Owner,
            0x0006_0001 => SentFor,
            0x0006_0200 => Delegate,
            0x0003_0006 => DateStart,
            0x0003_0007 => DateEnd,
            0x0005_0008 => AidOwner,
            0x0004_0090 => RequestRes,
            _ => return Err(Error::InvalidMessageId(id)),
        })
    }
}

/// Attachment level attribute IDs.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum AttachAttrId {
    Data,
    Title,
    MetaFile,
    CreateDate,
    ModifyDate,
    TransportFilename,
    AttachRendData,
    Attachment,
}

impl AttachAttrId {
    pub(crate) fn from_u32(id: u32) -> Result<Self, Error> {
        use AttachAttrId::*;
        Ok(match id {
            0x0006_800F => Data,
            0x0001_8010 => Title,
            0x0006_8011 => MetaFile,
            0x0003_8012 => CreateDate,
            0x0003_8013 => ModifyDate,
            0x0006_9001 => TransportFilename,
            0x0006_9002 => AttachRendData,
            0x0006_9005 => Attachment,
            _ => return Err(Error::InvalidAttachAttr(id)),
        })
    }
}
