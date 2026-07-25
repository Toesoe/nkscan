//! Additional Sense Code / Additional Sense Code Qualifier, decoded into a named condition
//!
//! Scoped to every entry whose device-type column includes `S` (scanner
//! device) - the full table has hundreds of entries for device types this
//! crate will never talk to. Anything outside that set decodes to `Other`

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AdditionalSenseCode {
    BeginningOfPartitionMediumDetected,
    ChangedOperatingDefinition,
    CommandPhaseError,
    CommandSequenceError,
    CommandsClearedByAnotherInitiator,
    CopyCannotExecuteSinceHostCannotDisconnect,
    DataPhaseError,
    /// ASCQ is the failing component ID (80h-FFh), not a fixed value
    DiagnosticFailureOnComponent(u8),
    EndOfDataDetected,
    EndOfPartitionMediumDetected,
    ErrorLogOverflow,
    ErrorTooLongToCorrect,
    IoProcessTerminated,
    InitiatorDetectedErrorMessageReceived,
    InquiryDataHasChanged,
    InternalTargetFailure,
    InvalidBitsInIdentifyMessage,
    InvalidCombinationOfWindowsSpecified,
    InvalidCommandOperationCode,
    InvalidFieldInCdb,
    InvalidFieldInParameterList,
    InvalidMessageError,
    LampFailure,
    LogCounterAtMaximum,
    LogException,
    LogListCodesExhausted,
    LogParametersChanged,
    LogicalUnitCommunicationFailure,
    LogicalUnitCommunicationParityError,
    LogicalUnitCommunicationTimeOut,
    LogicalUnitDoesNotRespondToSelection,
    LogicalUnitFailedSelfConfiguration,
    LogicalUnitHasNotSelfConfiguredYet,
    LogicalUnitIsInProcessOfBecomingReady,
    LogicalUnitNotReadyCauseNotReportable,
    LogicalUnitNotReadyInitializingCommandRequired,
    LogicalUnitNotReadyManualInterventionRequired,
    LogicalUnitNotSupported,
    MechanicalPositioningError,
    MediaLoadOrEjectFailed,
    MediumNotPresent,
    MessageError,
    MicrocodeHasBeenChanged,
    ModeParametersChanged,
    MultiplePeripheralDevicesSelected,
    MultipleReadErrors,
    NoAdditionalSenseInformation,
    NotReadyToReadyTransitionMediumMayHaveChanged,
    OperatorRequestOrStateChangeInputUnspecified,
    OutOfFocus,
    OverlappedCommandsAttempted,
    ParameterListLengthError,
    ParameterNotSupported,
    ParameterValueInvalid,
    ParametersChanged,
    PeripheralDeviceWriteFault,
    PositionPastBeginningOfMedium,
    PositionPastEndOfMedium,
    PowerOnResetOrBusDeviceResetOccurred,
    RandomPositioningError,
    ReadPastBeginningOfMedium,
    ReadPastEndOfMedium,
    ReadRetriesExhausted,
    RecordedEntityNotFound,
    RecoveredDataWithNoErrorCorrectionApplied,
    RecoveredDataWithRetries,
    RoundedParameter,
    SavingParametersNotSupported,
    ScanHeadPositioningError,
    ScsiParityError,
    SelectOrReselectFailure,
    SynchronousDataTransferError,
    TargetOperatingConditionsHaveChanged,
    ThresholdConditionMet,
    ThresholdParametersNotSupported,
    TooManyWindowsSpecified,
    UnableToAcquireVideo,
    UnrecoveredReadError,
    UnsuccessfulSoftReset,
    VideoAcquisitionError,
    WriteError,
    /// Not one of the scanner-relevant codes above; the raw `(ASC, ASCQ)`
    Other(u8, u8),
}

impl AdditionalSenseCode {
    pub fn from_asc_ascq(asc: u8, ascq: u8) -> Self {
        use AdditionalSenseCode::*;
        match (asc, ascq) {
            (0x00, 0x04) => BeginningOfPartitionMediumDetected,
            (0x3F, 0x02) => ChangedOperatingDefinition,
            (0x4A, 0x00) => CommandPhaseError,
            (0x2C, 0x00) => CommandSequenceError,
            (0x2F, 0x00) => CommandsClearedByAnotherInitiator,
            (0x2B, 0x00) => CopyCannotExecuteSinceHostCannotDisconnect,
            (0x4B, 0x00) => DataPhaseError,
            (0x40, ascq @ 0x80..=0xFF) => DiagnosticFailureOnComponent(ascq),
            (0x00, 0x05) => EndOfDataDetected,
            (0x00, 0x02) => EndOfPartitionMediumDetected,
            (0x0A, 0x00) => ErrorLogOverflow,
            (0x11, 0x02) => ErrorTooLongToCorrect,
            (0x00, 0x06) => IoProcessTerminated,
            (0x48, 0x00) => InitiatorDetectedErrorMessageReceived,
            (0x3F, 0x03) => InquiryDataHasChanged,
            (0x44, 0x00) => InternalTargetFailure,
            (0x3D, 0x00) => InvalidBitsInIdentifyMessage,
            (0x2C, 0x02) => InvalidCombinationOfWindowsSpecified,
            (0x20, 0x00) => InvalidCommandOperationCode,
            (0x24, 0x00) => InvalidFieldInCdb,
            (0x26, 0x00) => InvalidFieldInParameterList,
            (0x49, 0x00) => InvalidMessageError,
            (0x60, 0x00) => LampFailure,
            (0x5B, 0x02) => LogCounterAtMaximum,
            (0x5B, 0x00) => LogException,
            (0x5B, 0x03) => LogListCodesExhausted,
            (0x2A, 0x02) => LogParametersChanged,
            (0x08, 0x00) => LogicalUnitCommunicationFailure,
            (0x08, 0x02) => LogicalUnitCommunicationParityError,
            (0x08, 0x01) => LogicalUnitCommunicationTimeOut,
            (0x05, 0x00) => LogicalUnitDoesNotRespondToSelection,
            (0x4C, 0x00) => LogicalUnitFailedSelfConfiguration,
            (0x3E, 0x00) => LogicalUnitHasNotSelfConfiguredYet,
            (0x04, 0x01) => LogicalUnitIsInProcessOfBecomingReady,
            (0x04, 0x00) => LogicalUnitNotReadyCauseNotReportable,
            (0x04, 0x02) => LogicalUnitNotReadyInitializingCommandRequired,
            (0x04, 0x03) => LogicalUnitNotReadyManualInterventionRequired,
            (0x25, 0x00) => LogicalUnitNotSupported,
            (0x15, 0x01) => MechanicalPositioningError,
            (0x53, 0x00) => MediaLoadOrEjectFailed,
            (0x3A, 0x00) => MediumNotPresent,
            (0x43, 0x00) => MessageError,
            (0x3F, 0x01) => MicrocodeHasBeenChanged,
            (0x2A, 0x01) => ModeParametersChanged,
            (0x07, 0x00) => MultiplePeripheralDevicesSelected,
            (0x11, 0x03) => MultipleReadErrors,
            (0x00, 0x00) => NoAdditionalSenseInformation,
            (0x28, 0x00) => NotReadyToReadyTransitionMediumMayHaveChanged,
            (0x5A, 0x00) => OperatorRequestOrStateChangeInputUnspecified,
            (0x61, 0x02) => OutOfFocus,
            (0x4E, 0x00) => OverlappedCommandsAttempted,
            (0x1A, 0x00) => ParameterListLengthError,
            (0x26, 0x01) => ParameterNotSupported,
            (0x26, 0x02) => ParameterValueInvalid,
            (0x2A, 0x00) => ParametersChanged,
            (0x03, 0x00) => PeripheralDeviceWriteFault,
            (0x3B, 0x0C) => PositionPastBeginningOfMedium,
            (0x3B, 0x0B) => PositionPastEndOfMedium,
            (0x29, 0x00) => PowerOnResetOrBusDeviceResetOccurred,
            (0x15, 0x00) => RandomPositioningError,
            (0x3B, 0x0A) => ReadPastBeginningOfMedium,
            (0x3B, 0x09) => ReadPastEndOfMedium,
            (0x11, 0x01) => ReadRetriesExhausted,
            (0x14, 0x00) => RecordedEntityNotFound,
            (0x17, 0x00) => RecoveredDataWithNoErrorCorrectionApplied,
            (0x17, 0x01) => RecoveredDataWithRetries,
            (0x37, 0x00) => RoundedParameter,
            (0x39, 0x00) => SavingParametersNotSupported,
            (0x62, 0x00) => ScanHeadPositioningError,
            (0x47, 0x00) => ScsiParityError,
            (0x45, 0x00) => SelectOrReselectFailure,
            (0x1B, 0x00) => SynchronousDataTransferError,
            (0x3F, 0x00) => TargetOperatingConditionsHaveChanged,
            (0x5B, 0x01) => ThresholdConditionMet,
            (0x26, 0x03) => ThresholdParametersNotSupported,
            (0x2C, 0x01) => TooManyWindowsSpecified,
            (0x61, 0x01) => UnableToAcquireVideo,
            (0x11, 0x00) => UnrecoveredReadError,
            (0x46, 0x00) => UnsuccessfulSoftReset,
            (0x61, 0x00) => VideoAcquisitionError,
            (0x0C, 0x00) => WriteError,
            (asc, ascq) => Other(asc, ascq),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_code_decodes_to_named_variant() {
        assert_eq!(
            AdditionalSenseCode::from_asc_ascq(0x37, 0x00),
            AdditionalSenseCode::RoundedParameter
        );
    }

    #[test]
    fn diagnostic_failure_carries_component_id() {
        assert_eq!(
            AdditionalSenseCode::from_asc_ascq(0x40, 0x9F),
            AdditionalSenseCode::DiagnosticFailureOnComponent(0x9F)
        );
    }

    #[test]
    fn unrecognized_code_falls_back_to_other() {
        assert_eq!(
            AdditionalSenseCode::from_asc_ascq(0x3F, 0x04),
            AdditionalSenseCode::Other(0x3F, 0x04)
        );
    }
}
