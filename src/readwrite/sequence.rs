// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{AuthorisationKind, DataAuthKind, Type};
use crate::{
    Error, Response, SData as Sequence, SDataAddress as Address, SDataEntry as Entry,
    SDataIndex as Index, SDataOwner as Owner, SDataPrivPermissions as PrivatePermissions,
    SDataPubPermissions as PublicPermissions, SDataUser as User, SDataWriteOp as WriteOp, XorName,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};

/// TODO: docs
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub enum SequenceRead {
    /// Get Sequence from the network.
    Get(Address),
    /// Get a range of entries from an Sequence object on the network.
    GetRange {
        /// Sequence address.
        address: Address,
        /// Range of entries to fetch.
        ///
        /// For example, get 10 last entries:
        /// range: (Index::FromEnd(10), Index::FromEnd(0))
        ///
        /// Get all entries:
        /// range: (Index::FromStart(0), Index::FromEnd(0))
        ///
        /// Get first 5 entries:
        /// range: (Index::FromStart(0), Index::FromStart(5))
        range: (Index, Index),
    },
    /// Get last entry from the Sequence.
    GetLastEntry(Address),
    /// List all current users permissions.
    GetPermissions(Address),
    /// Get current permissions for a specified user(s).
    GetUserPermissions {
        /// Sequence address.
        address: Address,
        /// User to get permissions for.
        user: User,
    },
    /// Get current owner.
    GetOwner(Address),
}

/// TODO: docs
#[allow(clippy::large_enum_variant)]
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub enum SequenceWrite {
    /// Create a new Sequence on the network.
    New(Sequence),
    /// Edit the Sequence (insert/remove entry).
    Edit(WriteOp<Entry>),
    /// Delete a private Sequence.
    ///
    /// This operation MUST return an error if applied to public Sequence. Only the current
    /// owner(s) can perform this action.
    Delete(Address),
    /// Set a new owner. Only the current owner(s) can perform this action.
    SetOwner(WriteOp<Owner>),
    /// Set new permissions for public Sequence.
    SetPubPermissions(WriteOp<PublicPermissions>),
    /// Set new permissions for private Sequence.
    SetPrivPermissions(WriteOp<PrivatePermissions>),
}

impl SequenceRead {
    /// Get the `Type` of this request.
    pub fn get_type(&self) -> Type {
        use SequenceRead::*;
        match *self {
            Get(address)
            | GetRange { address, .. }
            | GetLastEntry(address)
            | GetPermissions(address)
            | GetUserPermissions { address, .. }
            | GetOwner(address) => {
                if address.is_pub() {
                    Type::PublicRead
                } else {
                    Type::PrivateRead
                }
            }
        }
    }

    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error_response(&self, error: Error) -> Response {
        use SequenceRead::*;
        match *self {
            Get(_) => Response::GetSData(Err(error)),
            GetRange { .. } => Response::GetSDataRange(Err(error)),
            GetLastEntry(_) => Response::GetSDataLastEntry(Err(error)),
            GetPermissions(_) => Response::GetSDataPermissions(Err(error)),
            GetUserPermissions { .. } => Response::GetSDataUserPermissions(Err(error)),
            GetOwner(_) => Response::GetSDataOwner(Err(error)),
        }
    }

    /// Returns the access categorisation of the request.
    pub fn authorisation_kind(&self) -> AuthorisationKind {
        use SequenceRead::*;
        match *self {
            Get(address)
            | GetRange { address, .. }
            | GetLastEntry(address)
            | GetPermissions(address)
            | GetUserPermissions { address, .. }
            | GetOwner(address) => {
                if address.is_pub() {
                    AuthorisationKind::Data(DataAuthKind::PublicRead)
                } else {
                    AuthorisationKind::Data(DataAuthKind::PrivateRead)
                }
            }
        }
    }

    /// Returns the address of the destination for request.
    pub fn dst_address(&self) -> Option<Cow<XorName>> {
        use SequenceRead::*;
        match self {
            Get(ref address) | GetRange { ref address, .. } | GetLastEntry(ref address) => {
                Some(Cow::Borrowed(address.name()))
            }
            GetPermissions(ref address)
            | GetUserPermissions { ref address, .. }
            | GetOwner(ref address) => Some(Cow::Borrowed(address.name())),
        }
    }
}

impl fmt::Debug for SequenceRead {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use SequenceRead::*;
        write!(
            formatter,
            "Request::{}",
            match *self {
                Get(_) => "GetSequence",
                GetRange { .. } => "GetSequenceRange",
                GetLastEntry(_) => "GetSequenceLastEntry",
                GetPermissions { .. } => "GetSequencePermissions",
                GetUserPermissions { .. } => "GetUserPermissions",
                GetOwner { .. } => "GetOwner",
            }
        )
    }
}

impl SequenceWrite {
    /// Get the `Type` of this request.
    pub fn get_type(&self) -> Type {
        Type::Write
    }

    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error_response(&self, error: Error) -> Response {
        Response::Write(Err(error))
    }

    /// Returns the access categorisation of the request.
    pub fn authorisation_kind(&self) -> AuthorisationKind {
        AuthorisationKind::Data(DataAuthKind::Write)
    }

    /// Returns the address of the destination for request.
    pub fn dst_address(&self) -> Option<Cow<XorName>> {
        use SequenceWrite::*;
        match self {
            New(ref data) => Some(Cow::Borrowed(data.name())),
            Delete(ref address) => Some(Cow::Borrowed(address.name())),
            SetPubPermissions(ref op) => Some(Cow::Borrowed(op.address.name())),
            SetPrivPermissions(ref op) => Some(Cow::Borrowed(op.address.name())),
            SetOwner(ref op) => Some(Cow::Borrowed(op.address.name())),
            Edit(ref op) => Some(Cow::Borrowed(op.address.name())),
        }
    }
}

impl fmt::Debug for SequenceWrite {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use SequenceWrite::*;
        write!(
            formatter,
            "Request::{}",
            match *self {
                New(_) => "NewSequence",
                Delete(_) => "DeleteSequence",
                SetPubPermissions(_) => "SetPublicPermissions",
                SetPrivPermissions(_) => "SetPrivatePermissions",
                SetOwner(_) => "SetOwner",
                Edit(_) => "EditSequence",
            }
        )
    }
}
