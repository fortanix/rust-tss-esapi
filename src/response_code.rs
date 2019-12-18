// Copyright (c) 2019, Arm Limited, All Rights Reserved
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may
// not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//          http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use crate::tss2_esys::TSS2_RC;
use bitfield::bitfield;

pub type Result<T> = std::result::Result<T, Error>;

bitfield! {
    pub struct ResponseCode(TSS2_RC);
    impl Debug;
    format_selector, _: 7;
}

bitfield! {
    #[derive(PartialEq, Copy, Clone)]
    pub struct FormatZeroResponseCode(TSS2_RC);
    impl Debug;
    error_number, _: 6, 0;
    format_selector, _: 7;
    version, _: 8;
    tcg_vendor_indicator, _: 10;
    severity, _: 11;
}

bitfield! {
    #[derive(PartialEq, Copy, Clone)]
    pub struct FormatOneResponseCode(TSS2_RC);
    impl Debug;
    error_number, _: 5, 0;
    parameter, _: 6;
    format_selector, _: 7;
    number, _: 11, 8;
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Tss2ResponseCode {
    Success,
    FormatZero(FormatZeroResponseCode),
    FormatOne(FormatOneResponseCode),
}

impl Tss2ResponseCode {
    pub(crate) fn from_tss_rc(response_code: TSS2_RC) -> Self {
        if response_code == 0 {
            Tss2ResponseCode::Success
        } else if ResponseCode(response_code).format_selector() {
            // The response code is in Format-One.
            Tss2ResponseCode::FormatOne(FormatOneResponseCode(response_code))
        } else {
            // The response code is in Format-Zero.
            Tss2ResponseCode::FormatZero(FormatZeroResponseCode(response_code))
        }
    }

    pub fn is_success(self) -> bool {
        self == Tss2ResponseCode::Success
    }

    fn is_warning(self) -> bool {
        match self {
            Tss2ResponseCode::Success => false,
            Tss2ResponseCode::FormatZero(rc) => rc.severity(),
            Tss2ResponseCode::FormatOne(_) => false,
        }
    }

    fn error_number(self) -> u32 {
        match self {
            Tss2ResponseCode::Success => 0,
            Tss2ResponseCode::FormatZero(rc) => rc.error_number(),
            Tss2ResponseCode::FormatOne(rc) => rc.error_number(),
        }
    }

    fn get_associated_number_message(self) -> String {
        if let Tss2ResponseCode::FormatOne(rc) = self {
            if rc.parameter() {
                format!("associated with parameter number {}", rc.number())
            } else if rc.number() <= 0b0111 {
                format!("associated with handle number {}", rc.number())
            } else {
                format!("associated with session number {}", rc.number() - 8)
            }
        } else {
            String::from("no associated message")
        }
    }

    pub fn kind(self) -> Option<Tss2ResponseCodeKind> {
        match self {
            Tss2ResponseCode::Success => Some(Tss2ResponseCodeKind::Success),
            Tss2ResponseCode::FormatZero(rc) => {
                if rc.tcg_vendor_indicator() {
                    Some(Tss2ResponseCodeKind::TpmVendorSpecific)
                } else if self.is_warning() {
                    // Warnings
                    match self.error_number() {
                        0x001 => Some(Tss2ResponseCodeKind::ContextGap),
                        0x002 => Some(Tss2ResponseCodeKind::ObjectMemory),
                        0x003 => Some(Tss2ResponseCodeKind::SessionMemory),
                        0x004 => Some(Tss2ResponseCodeKind::Memory),
                        0x005 => Some(Tss2ResponseCodeKind::SessionHandles),
                        0x006 => Some(Tss2ResponseCodeKind::ObjectHandles),
                        0x007 => Some(Tss2ResponseCodeKind::Locality),
                        0x008 => Some(Tss2ResponseCodeKind::Yielded),
                        0x009 => Some(Tss2ResponseCodeKind::Canceled),
                        0x00A => Some(Tss2ResponseCodeKind::Testing),
                        0x010 => Some(Tss2ResponseCodeKind::ReferenceH0),
                        0x011 => Some(Tss2ResponseCodeKind::ReferenceH1),
                        0x012 => Some(Tss2ResponseCodeKind::ReferenceH2),
                        0x013 => Some(Tss2ResponseCodeKind::ReferenceH3),
                        0x014 => Some(Tss2ResponseCodeKind::ReferenceH4),
                        0x015 => Some(Tss2ResponseCodeKind::ReferenceH5),
                        0x016 => Some(Tss2ResponseCodeKind::ReferenceH6),
                        0x018 => Some(Tss2ResponseCodeKind::ReferenceS0),
                        0x019 => Some(Tss2ResponseCodeKind::ReferenceS1),
                        0x01A => Some(Tss2ResponseCodeKind::ReferenceS2),
                        0x01B => Some(Tss2ResponseCodeKind::ReferenceS3),
                        0x01C => Some(Tss2ResponseCodeKind::ReferenceS4),
                        0x01D => Some(Tss2ResponseCodeKind::ReferenceS5),
                        0x01E => Some(Tss2ResponseCodeKind::ReferenceS6),
                        0x020 => Some(Tss2ResponseCodeKind::NvRate),
                        0x021 => Some(Tss2ResponseCodeKind::Lockout),
                        0x022 => Some(Tss2ResponseCodeKind::Retry),
                        0x023 => Some(Tss2ResponseCodeKind::NvUnavailable),
                        _ => None,
                    }
                } else {
                    // Errors
                    match self.error_number() {
                        0x000 => Some(Tss2ResponseCodeKind::Initialize),
                        0x001 => Some(Tss2ResponseCodeKind::Failure),
                        0x003 => Some(Tss2ResponseCodeKind::Sequence),
                        0x00B => Some(Tss2ResponseCodeKind::Private),
                        0x019 => Some(Tss2ResponseCodeKind::Hmac),
                        0x020 => Some(Tss2ResponseCodeKind::Disabled),
                        0x021 => Some(Tss2ResponseCodeKind::Exclusive),
                        0x024 => Some(Tss2ResponseCodeKind::AuthType),
                        0x025 => Some(Tss2ResponseCodeKind::AuthMissing),
                        0x026 => Some(Tss2ResponseCodeKind::Policy),
                        0x027 => Some(Tss2ResponseCodeKind::Pcr),
                        0x028 => Some(Tss2ResponseCodeKind::PcrChanged),
                        0x02D => Some(Tss2ResponseCodeKind::Upgrade),
                        0x02E => Some(Tss2ResponseCodeKind::TooManyContexts),
                        0x02F => Some(Tss2ResponseCodeKind::AuthUnavailable),
                        0x030 => Some(Tss2ResponseCodeKind::Reboot),
                        0x031 => Some(Tss2ResponseCodeKind::Unbalanced),
                        0x042 => Some(Tss2ResponseCodeKind::CommandSize),
                        0x043 => Some(Tss2ResponseCodeKind::CommandCode),
                        0x044 => Some(Tss2ResponseCodeKind::AuthSize),
                        0x045 => Some(Tss2ResponseCodeKind::AuthContext),
                        0x046 => Some(Tss2ResponseCodeKind::NvRange),
                        0x047 => Some(Tss2ResponseCodeKind::NvSize),
                        0x048 => Some(Tss2ResponseCodeKind::NvLocked),
                        0x049 => Some(Tss2ResponseCodeKind::NvAuthorization),
                        0x04A => Some(Tss2ResponseCodeKind::NvUninitialized),
                        0x04B => Some(Tss2ResponseCodeKind::NvSpace),
                        0x04C => Some(Tss2ResponseCodeKind::NvDefined),
                        0x050 => Some(Tss2ResponseCodeKind::BadContext),
                        0x051 => Some(Tss2ResponseCodeKind::CpHash),
                        0x052 => Some(Tss2ResponseCodeKind::Parent),
                        0x053 => Some(Tss2ResponseCodeKind::NeedsTest),
                        0x054 => Some(Tss2ResponseCodeKind::NoResult),
                        0x055 => Some(Tss2ResponseCodeKind::Sensitive),
                        _ => None,
                    }
                }
            }
            Tss2ResponseCode::FormatOne(_) => match self.error_number() {
                0x001 => Some(Tss2ResponseCodeKind::Asymmetric),
                0x002 => Some(Tss2ResponseCodeKind::Attributes),
                0x003 => Some(Tss2ResponseCodeKind::Hash),
                0x004 => Some(Tss2ResponseCodeKind::Value),
                0x005 => Some(Tss2ResponseCodeKind::Hierarchy),
                0x007 => Some(Tss2ResponseCodeKind::KeySize),
                0x008 => Some(Tss2ResponseCodeKind::Mgf),
                0x009 => Some(Tss2ResponseCodeKind::Mode),
                0x00A => Some(Tss2ResponseCodeKind::Type),
                0x00B => Some(Tss2ResponseCodeKind::Handle),
                0x00C => Some(Tss2ResponseCodeKind::Kdf),
                0x00D => Some(Tss2ResponseCodeKind::Range),
                0x00E => Some(Tss2ResponseCodeKind::AuthFail),
                0x00F => Some(Tss2ResponseCodeKind::Nonce),
                0x010 => Some(Tss2ResponseCodeKind::Pp),
                0x012 => Some(Tss2ResponseCodeKind::Scheme),
                0x015 => Some(Tss2ResponseCodeKind::Size),
                0x016 => Some(Tss2ResponseCodeKind::Symmetric),
                0x017 => Some(Tss2ResponseCodeKind::Tag),
                0x018 => Some(Tss2ResponseCodeKind::Selector),
                0x01A => Some(Tss2ResponseCodeKind::Insufficient),
                0x01B => Some(Tss2ResponseCodeKind::Signature),
                0x01C => Some(Tss2ResponseCodeKind::Key),
                0x01D => Some(Tss2ResponseCodeKind::PolicyFail),
                0x01F => Some(Tss2ResponseCodeKind::Integrity),
                0x020 => Some(Tss2ResponseCodeKind::Ticket),
                0x021 => Some(Tss2ResponseCodeKind::ReservedBits),
                0x022 => Some(Tss2ResponseCodeKind::BadAuth),
                0x023 => Some(Tss2ResponseCodeKind::Expired),
                0x024 => Some(Tss2ResponseCodeKind::PolicyCc),
                0x025 => Some(Tss2ResponseCodeKind::Binding),
                0x026 => Some(Tss2ResponseCodeKind::Curve),
                0x027 => Some(Tss2ResponseCodeKind::EccPoint),
                _ => None,
            },
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Tss2ResponseCodeKind {
    // FormatZero errors
    Success,
    TpmVendorSpecific,
    Initialize,
    Failure,
    Sequence,
    Private,
    Hmac,
    Disabled,
    Exclusive,
    AuthType,
    AuthMissing,
    Policy,
    Pcr,
    PcrChanged,
    Upgrade,
    TooManyContexts,
    AuthUnavailable,
    Reboot,
    Unbalanced,
    CommandSize,
    CommandCode,
    AuthSize,
    AuthContext,
    NvRange,
    NvSize,
    NvLocked,
    NvAuthorization,
    NvUninitialized,
    NvSpace,
    NvDefined,
    BadContext,
    CpHash,
    Parent,
    NeedsTest,
    NoResult,
    Sensitive,
    // FormatOne errors
    Asymmetric,
    Attributes,
    Hash,
    Value,
    Hierarchy,
    KeySize,
    Mgf,
    Mode,
    Type,
    Handle,
    Kdf,
    Range,
    AuthFail,
    Nonce,
    Pp,
    Scheme,
    Size,
    Symmetric,
    Tag,
    Selector,
    Insufficient,
    Signature,
    Key,
    PolicyFail,
    Integrity,
    Ticket,
    ReservedBits,
    BadAuth,
    Expired,
    PolicyCc,
    Binding,
    Curve,
    EccPoint,
    // Warnings
    ContextGap,
    ObjectMemory,
    SessionMemory,
    Memory,
    SessionHandles,
    ObjectHandles,
    Locality,
    Yielded,
    Canceled,
    Testing,
    ReferenceH0,
    ReferenceH1,
    ReferenceH2,
    ReferenceH3,
    ReferenceH4,
    ReferenceH5,
    ReferenceH6,
    ReferenceS0,
    ReferenceS1,
    ReferenceS2,
    ReferenceS3,
    ReferenceS4,
    ReferenceS5,
    ReferenceS6,
    NvRate,
    Lockout,
    Retry,
    NvUnavailable,
}

impl std::fmt::Display for Tss2ResponseCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = self.kind();
        if kind.is_none() {
            return write!(f, "response code not recognized");
        }
        match self.kind().unwrap() { // should not panic, given the check above
            Tss2ResponseCodeKind::Success => write!(f, "success"),
            Tss2ResponseCodeKind::TpmVendorSpecific => write!(f, "vendor specific error: {}", self.error_number()),
            // Format Zero
            Tss2ResponseCodeKind::Initialize => write!(f, "TPM not initialized by TPM2_Startup or already initialized"),
            Tss2ResponseCodeKind::Failure => write!(f, "commands not being accepted because of a TPM failure. NOTE: This may be returned by TPM2_GetTestResult() as the testResultparameter"),
            Tss2ResponseCodeKind::Sequence => write!(f, "improper use of a sequence handle"),
            Tss2ResponseCodeKind::Private => write!(f, "not currently used"),
            Tss2ResponseCodeKind::Hmac => write!(f, "not currently used"),
            Tss2ResponseCodeKind::Disabled => write!(f, "the command is disabled"),
            Tss2ResponseCodeKind::Exclusive => write!(f, "command failed because audit sequence required exclusivity"),
            Tss2ResponseCodeKind::AuthType => write!(f, "authorization handle is not correct for command"),
            Tss2ResponseCodeKind::AuthMissing => write!(f, "command requires an authorization session for handle and it is not present"),
            Tss2ResponseCodeKind::Policy => write!(f, "policy failure in math operation or an invalid authPolicy value"),
            Tss2ResponseCodeKind::Pcr => write!(f, "PCR check fail"),
            Tss2ResponseCodeKind::PcrChanged => write!(f, "PCR have changed since checked"),
            Tss2ResponseCodeKind::Upgrade => write!(f, "for all commands other than TPM2_FieldUpgradeData(), this code indicates that the TPM is in field upgrade mode; for TPM2_FieldUpgradeData(), this code indicates that the TPM is not in field upgrade mode"),
            Tss2ResponseCodeKind::TooManyContexts => write!(f, "context ID counter is at maximum"),
            Tss2ResponseCodeKind::AuthUnavailable => write!(f, "authValue or authPolicy is not available for selected entity"),
            Tss2ResponseCodeKind::Reboot => write!(f, "a _TPM_Init and Startup(CLEAR) is required before the TPM can resume operation"),
            Tss2ResponseCodeKind::Unbalanced => write!(f, "the protection algorithms (hash and symmetric) are not reasonably balanced. The digest size of the hash must be larger than the key size of the symmetric algorithm"),
            Tss2ResponseCodeKind::CommandSize => write!(f, "command commandSizevalue is inconsistent with contents of the command buffer; either the size is not the same as the octets loaded by the hardware interface layer or the value is not large enough to hold a command header"),
            Tss2ResponseCodeKind::CommandCode => write!(f, "command code not supported"),
            Tss2ResponseCodeKind::AuthSize => write!(f, "the value of authorizationSizeis out of range or the number of octets in the Authorization Area is greater than required"),
            Tss2ResponseCodeKind::AuthContext => write!(f, "use of an authorization session with a context command or another command that cannot have an authorization session"),
            Tss2ResponseCodeKind::NvRange => write!(f, "NV offset+size is out of range"),
            Tss2ResponseCodeKind::NvSize => write!(f, "Requested allocation size is larger than allowed"),
            Tss2ResponseCodeKind::NvLocked => write!(f, "NV access locked"),
            Tss2ResponseCodeKind::NvAuthorization => write!(f, "NV access authorization fails in command actions (this failure does not affect lockout.action)"),
            Tss2ResponseCodeKind::NvUninitialized => write!(f, "an NV Index is used before being initialized or the state saved by TPM2_Shutdown(STATE) could not be restored"),
            Tss2ResponseCodeKind::NvSpace => write!(f, "insufficient space for NV allocation"),
            Tss2ResponseCodeKind::NvDefined => write!(f, "NV Index or persistent object already defined"),
            Tss2ResponseCodeKind::BadContext => write!(f, "context in TPM2_ContextLoad() is not valid"),
            Tss2ResponseCodeKind::CpHash => write!(f, "cpHash value already set or not correct for use"),
            Tss2ResponseCodeKind::Parent => write!(f, "handle for parent is not a valid parent"),
            Tss2ResponseCodeKind::NeedsTest => write!(f, "some function needs testing."),
            Tss2ResponseCodeKind::NoResult => write!(f, "returned when an internal function cannot process a request due to an unspecified problem. This code is usually related to invalid parameters that are not properly filtered by the input unmarshaling code."),
            Tss2ResponseCodeKind::Sensitive => write!(f, "the sensitive area did not unmarshal correctly after decryption – this code is used in lieu of the other unmarshaling errors so that an attacker cannot determine where the unmarshaling error occurred"),
            // Warnings
            Tss2ResponseCodeKind::ContextGap => write!(f, "gap for context ID is too large"),
            Tss2ResponseCodeKind::ObjectMemory => write!(f, "out of memory for object contexts"),
            Tss2ResponseCodeKind::SessionMemory => write!(f, "out of memory for session contexts"),
            Tss2ResponseCodeKind::Memory => write!(f, "out of shared object/session memory or need space for internal operations"),
            Tss2ResponseCodeKind::SessionHandles => write!(f, "out of session handles – a session must be flushed before a new session may be created"),
            Tss2ResponseCodeKind::ObjectHandles => write!(f, "out of object handles – the handle space for objects is depleted and a reboot is required. NOTE 1: This cannot occur on the reference implementation. NOTE 2: There is no reason why an implementation would implement a design that would delete handle space. Platform specifications are encouraged to forbid it."),
            Tss2ResponseCodeKind::Locality => write!(f, "bad locality"),
            Tss2ResponseCodeKind::Yielded => write!(f, "the TPM has suspended operation on the command; forward progress was made and the command may be retried. See TPM 2.0 Part 1, “Multi-tasking.” NOTE: This cannot occur on the reference implementation."),
            Tss2ResponseCodeKind::Canceled => write!(f, "the command was canceled"),
            Tss2ResponseCodeKind::Testing => write!(f, "TPM is performing self-tests"),
            Tss2ResponseCodeKind::ReferenceH0 => write!(f, "the 1st handle in the handle area references a transient object or session that is not loaded"),
            Tss2ResponseCodeKind::ReferenceH1 => write!(f, "the 2nd handle in the handle area references a transient object or session that is not loaded"),
            Tss2ResponseCodeKind::ReferenceH2 => write!(f, "the 3rd handle in the handle area references a transient object or session that is not loaded"),
            Tss2ResponseCodeKind::ReferenceH3 => write!(f, "the 4th handle in the handle area references a transient object or session that is not loaded"),
            Tss2ResponseCodeKind::ReferenceH4 => write!(f, "the 5th handle in the handle area references a transient object or session that is not loaded"),
            Tss2ResponseCodeKind::ReferenceH5 => write!(f, "the 6th handle in the handle area references a transient object or session that is not loaded"),
            Tss2ResponseCodeKind::ReferenceH6 => write!(f, "the 7th handle in the handle area references a transient object or session that is not loaded"),
            Tss2ResponseCodeKind::ReferenceS0 => write!(f, "the 1st authorization session handle references a session that is not loaded"),
            Tss2ResponseCodeKind::ReferenceS1 => write!(f, "the 2nd authorization session handle references a session that is not loaded"),
            Tss2ResponseCodeKind::ReferenceS2 => write!(f, "the 3rd authorization session handle references a session that is not loaded"),
            Tss2ResponseCodeKind::ReferenceS3 => write!(f, "the 4th authorization session handle references a session that is not loaded"),
            Tss2ResponseCodeKind::ReferenceS4 => write!(f, "the 5th session handle references a session that is not loaded"),
            Tss2ResponseCodeKind::ReferenceS5 => write!(f, "the 6th session handle references a session that is not loaded"),
            Tss2ResponseCodeKind::ReferenceS6 => write!(f, "the 7th authorization session handle references a session that is not loaded"),
            Tss2ResponseCodeKind::NvRate => write!(f, "the TPM is rate-limiting accesses to prevent wearout of NV"),
            Tss2ResponseCodeKind::Lockout => write!(f, "authorizations for objects subject to DA protection are not allowed at this time because the TPM is in DA lockout mode"),
            Tss2ResponseCodeKind::Retry => write!(f, "the TPM was not able to start the command"),
            Tss2ResponseCodeKind::NvUnavailable => write!(f, "the command may require writing of NV and NV is not current accessible"),
            // Format-One
            Tss2ResponseCodeKind::Asymmetric => write!(f, "asymmetric algorithm not supported or not correct ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Attributes => write!(f, "inconsistent attributes ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Hash => write!(f, "hash algorithm not supported or not appropriate ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Value => write!(f, "value is out of range or is not correct for the context ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Hierarchy => write!(f, "hierarchy is not enabled or is not correct for the use ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::KeySize => write!(f, "key size is not supported ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Mgf => write!(f, "mask generation function not supported ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Mode => write!(f, "mode of operation not supported ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Type => write!(f, "the type of the value is not appropriate for the use ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Handle => write!(f, "the handle is not correct for the use ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Kdf => write!(f, "unsupported key derivation function or function not appropriate for use ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Range => write!(f, "value was out of allowed range. ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::AuthFail => write!(f, "the authorization HMAC check failed and DA counter incremented ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Nonce => write!(f, "invalid nonce size or nonce value mismatch ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Pp => write!(f, "authorization requires assertion of PP ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Scheme => write!(f, "unsupported or incompatible scheme ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Size => write!(f, "structure is the wrong size ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Symmetric => write!(f, "unsupported symmetric algorithm or key size, or not appropriate for instance ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Tag => write!(f, "incorrect structure tag ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Selector => write!(f, "union selector is incorrect ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Insufficient => write!(f, "the TPM was unable to unmarshal a value because there were not enough octets in the input buffer ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Signature => write!(f, "the signature is not valid ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Key => write!(f, "key fields are not compatible with the selected use ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::PolicyFail => write!(f, "a policy check failed ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Integrity => write!(f, "integrity check failed ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Ticket => write!(f, "invalid ticket  ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::ReservedBits => write!(f, "reserved bits not set to zero as required ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::BadAuth => write!(f, "authorization failure without DA implications ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Expired => write!(f, "the policy has expired ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::PolicyCc => write!(f, "the command Code in the policy is not the command Code of the command or the command code in a policy command references a command that is not implemented ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Binding => write!(f, "public and sensitive portions of an object are not cryptographically bound ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::Curve => write!(f, "curve not supported ({})", self.get_associated_number_message()),
            Tss2ResponseCodeKind::EccPoint => write!(f, "point is not on the required curve ({})", self.get_associated_number_message()),
        }
    }
}

impl From<TSS2_RC> for Tss2ResponseCode {
    fn from(rc: TSS2_RC) -> Self {
        Tss2ResponseCode::from_tss_rc(rc)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Error {
    WrapperError(WrapperErrorKind),
    Tss2Error(Tss2ResponseCode),
}

impl Error {
    pub(crate) fn from_tss_rc(rc: TSS2_RC) -> Self {
        Error::Tss2Error(Tss2ResponseCode::from_tss_rc(rc))
    }

    pub(crate) fn local_error(kind: WrapperErrorKind) -> Self {
        Error::WrapperError(kind)
    }

    pub fn is_success(self) -> bool {
        if let Error::Tss2Error(tss2_rc) = self {
            tss2_rc.is_success()
        } else {
            false
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::WrapperError(e) => e.fmt(f),
            Error::Tss2Error(e) => e.fmt(f),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum WrapperErrorKind {
    WrongParamSize,
    ParamsMissing,
    InconsistentParams,
}

impl std::fmt::Display for WrapperErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WrapperErrorKind::WrongParamSize => {
                write!(f, "parameter provided is of the wrong size")
            }
            WrapperErrorKind::ParamsMissing => {
                write!(f, "some of the required parameters were not provided")
            }
            WrapperErrorKind::InconsistentParams => write!(
                f,
                "the provided parameters have inconsistent values or variants"
            ),
        }
    }
}