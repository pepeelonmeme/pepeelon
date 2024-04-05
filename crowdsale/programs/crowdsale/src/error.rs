use anchor_lang::prelude::error_code;

#[error_code]
pub enum CrowdSaleError {
    #[msg("Invalid Start Time: Start time must be greater than current unix timestamp")]
    InvalidStartTime,

    #[msg("Invalid End Time: End time must be greater than start time")]
    InvalidEndTime,

    #[msg("Invalid Price Range: Max price must be greater than min price")]
    InvalidPriceRange,

    #[msg("Unauthorized: Only the authority can perform this action")]
    Unauthorized,

    #[msg("Invalid Time: Sale has not started yet")]
    SaleNotStart,

    #[msg("Invalid Time: Sale has ended")]
    SaleEnded,

    #[msg("Invalid Time: Sale has not ended")]
    SaleNotEnded,

    #[msg("Invalid Amount: Amount does not meet the minimum price requirement")]
    MinimumPriceNotMet,

    #[msg("Invalid Amount: Amount exceeds the maximum price allowed")]
    ExceedsMaximumPrice,

    #[msg("Invalid Amount: Amount not in range")]
    InvalidPrice,

    #[msg("Invalid Amount: Amount exceeds the supply")]
    ExceedsSupply,

    #[msg("Invalid Amount: Reach maximum amount can buy")]
    InvalidMaximumCanBuy,
}
