pub enum ErrorCode {
    LOK,
    NotApproved,
    InvalidUpdateConfigFlag,
    AccountLack,
    ClosePositionErr,

    ZeroMintAmount,

    InvaildTickIndex,
    TickInvaildOrder,
    TickLowerOverflow,
    TickUpperOverflow,
    TickAndSpacingNotMatch,
    InvalidTickArray,
    InvalidTickArrayBoundary,

    SqrtPriceLimitOverflow,
    // second inequality must be < because the price can never reach the price at the max tick
    SqrtPriceX64,

    // Liquidity Sub
    LiquiditySubValueErr,
    // Liquidity Add
    LiquidityAddValueErr,
    InvaildLiquidity,
    ForbidBothZeroForSupplyLiquidity,
    LiquidityInsufficient,

    /// swap errors
    // Non fungible position manager
    TransactionTooOld,
    PriceSlippageCheck,
    TooLittleOutputReceived,
    TooMuchInputPaid,
    ZeroAmountSpecified,
    InvalidInputPoolVault,
    TooSmallInputOrOutputAmount,
    NotEnoughTickArrayAccount,
    InvalidFirstTickArrayAccount,

    /// reward errors
    InvalidRewardIndex,
    FullRewardInfo,
    RewardTokenAlreadyInUse,
    ExceptPoolVaultMint,
    InvalidRewardInitParam,
    InvalidRewardDesiredAmount,
    InvalidRewardInputAccountNumber,
    InvalidRewardPeriod,
    NotApproveUpdateRewardEmissiones,
    UnInitializedRewardInfo,

    NotSupportMint,
    MissingTickArrayBitmapExtensionAccount,
    InsufficientLiquidityForDirection,
    MaxTokenOverflow,
    CalculateOverflow,
}
