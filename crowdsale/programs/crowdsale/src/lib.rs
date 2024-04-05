use anchor_lang::{
    prelude::*,
    solana_program::{clock::Clock, native_token::LAMPORTS_PER_SOL},
    system_program,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
mod constants;
mod errors;

use crate::{constants::*, errors::*};

declare_id!("98otN1KwVKtKaWHjsje2UqhCh4K86VNKTs6eZtK6UKe4");

#[program]
pub mod crowdsale {
    use super::*;

    pub fn create_crowdsale(ctx: Context<CreateCrowdsale>) -> Result<()> {
        let crowdsale_account = &mut ctx.accounts.crowdsale_account;

        crowdsale_account.authority = ctx.accounts.payer.key();

        Ok(())
    }

    pub fn setting_crowdsale(
        ctx: Context<SettingCrowdsale>,
        min_price: u64,
        max_price: u64,
        start: u64,
        end: u64,
        price: u64,
    ) -> Result<()> {
        let crowdsale_account = &mut ctx.accounts.crowdsale_account;
        let payer = &mut ctx.accounts.payer;
        let clock = Clock::get()?;

        if payer.key() != crowdsale_account.authority {
            return err!(CrowdSaleError::Unauthorized);
        }

        if start <= clock.unix_timestamp as u64 {
            return err!(CrowdSaleError::InvalidStartTime);
        }

        if end <= start {
            return err!(CrowdSaleError::InvalidEndTime);
        }

        crowdsale_account.start = start;
        crowdsale_account.end = end;
        crowdsale_account.min_price = min_price;
        crowdsale_account.max_price = max_price;
        crowdsale_account.price = price;

        Ok(())
    }

    pub fn deposite_token(ctx: Context<TokenSale>, amount: u64) -> Result<()> {
        let crowdsale_account = &mut ctx.accounts.crowdsale_account;
        let payer = &mut ctx.accounts.payer;

        if payer.key() != crowdsale_account.authority {
            return err!(CrowdSaleError::Unauthorized);
        }

        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.crowdsale_token_vault_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            amount,
        )?;

        crowdsale_account.supply += amount;

        Ok(())
    }

    pub fn buy_token(ctx: Context<BuyToken>, authority: Pubkey, amount: u64) -> Result<()> {
        let crowdsale_account = &mut ctx.accounts.crowdsale_account;
        let user_info_account = &mut ctx.accounts.user_info_account;
        let clock = Clock::get()?;

        if !is_sale_active(crowdsale_account, clock)? {
            return err!(CrowdSaleError::SaleNotStart);
        }

        if !is_price_valid(crowdsale_account, amount) {
            return err!(CrowdSaleError::InvalidPrice);
        }

        if crowdsale_account.authority != authority {
            return err!(CrowdSaleError::Unauthorized);
        }

        if user_info_account.sol_amount + amount > crowdsale_account.max_price {
            return err!(CrowdSaleError::InvalidMaximumCanBuy);
        }

        let reward_amount = calculate_reward_amount(crowdsale_account, amount)?;

        if reward_amount + crowdsale_account.sold_amount > crowdsale_account.supply {
            return err!(CrowdSaleError::ExceedsSupply);
        }

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.payer.to_account_info().clone(),
                to: crowdsale_account.to_account_info().clone(),
            },
        );
        system_program::transfer(cpi_context, amount)?;

        let bump = ctx.bumps.crowdsale_token_vault_account;
        let seed = &[CROWDSALE_TOKEN_VAULT_SEED, &[bump]];
        let signer = &[&seed[..]];

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.crowdsale_token_vault_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.crowdsale_token_vault_account.to_account_info(),
                },
                signer,
            ),
            reward_amount,
        )?;

        crowdsale_account.sold_amount += reward_amount;
        user_info_account.sol_amount += amount;
        user_info_account.bought_amount += reward_amount;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let crowdsale_account = &mut ctx.accounts.crowdsale_account;
        let payer = &mut ctx.accounts.payer;

        if payer.key() != crowdsale_account.authority {
            return err!(CrowdSaleError::Unauthorized);
        }

        **crowdsale_account
            .to_account_info()
            .try_borrow_mut_lamports()? -= amount;

        **payer.to_account_info().try_borrow_mut_lamports()? += amount;

        Ok(())
    }

    pub fn end_sales(ctx: Context<TokenSale>) -> Result<()> {
        let crowdsale_account = &mut ctx.accounts.crowdsale_account;
        let payer = &mut ctx.accounts.payer;
        let clock = Clock::get()?;

        if payer.key() != crowdsale_account.authority {
            return err!(CrowdSaleError::Unauthorized);
        }

        if crowdsale_account.end > clock.unix_timestamp as u64 {
            return err!(CrowdSaleError::SaleNotEnded);
        }

        let amount = crowdsale_account
            .supply
            .checked_sub(crowdsale_account.sold_amount)
            .unwrap();

        let bump = ctx.bumps.crowdsale_token_vault_account;
        let seed = &[CROWDSALE_TOKEN_VAULT_SEED, &[bump]];
        let signer = &[&seed[..]];

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.crowdsale_token_vault_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.crowdsale_token_vault_account.to_account_info(),
                },
                signer,
            ),
            amount,
        )?;

        crowdsale_account.sold_amount = crowdsale_account.supply;

        Ok(())
    }

}

#[derive(Accounts)]
pub struct CreateCrowdsale<'info> {
    #[account(
        init, 
        payer = payer, 
        space =8 + std::mem::size_of::<CrowdSale>(),
        seeds=[CROWDSALE_SEED, payer.key().as_ref()],
        bump    
    )]
    pub crowdsale_account: Account<'info, CrowdSale>,

    #[account(
        init_if_needed,
        seeds = [CROWDSALE_TOKEN_VAULT_SEED],
        bump,
        payer = payer, 
        token::mint = mint, 
        token::authority = crowdsale_token_vault_account
    )]
    pub crowdsale_token_vault_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SettingCrowdsale<'info> {
    #[account(
        mut,
        seeds=[CROWDSALE_SEED, payer.key().as_ref()],
        bump    
    )]
    pub crowdsale_account: Account<'info, CrowdSale>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TokenSale<'info> {
    #[account(
        mut,
        seeds=[CROWDSALE_SEED, payer.key().as_ref()],
        bump    
    )]
    pub crowdsale_account: Account<'info, CrowdSale>,

    #[account(
        init_if_needed,
        seeds = [CROWDSALE_TOKEN_VAULT_SEED],
        payer = payer,
        bump,
        token::mint = mint, 
        token::authority = crowdsale_token_vault_account
    )]
    pub crowdsale_token_vault_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(authority: Pubkey)]
pub struct BuyToken<'info> {
    #[account(
        mut,
        seeds=[CROWDSALE_SEED, authority.key().as_ref()],
        bump    
    )]
    pub crowdsale_account: Account<'info, CrowdSale>,

    #[account(
        mut,
        seeds = [CROWDSALE_TOKEN_VAULT_SEED],
        bump,
    )]
    pub crowdsale_token_vault_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        space=8 + std::mem::size_of::<UserInfo>(),
        seeds=[USER_INFO_SEED, payer.key().as_ref()],
        bump,
        payer = payer
    )]
    pub user_info_account: Account<'info, UserInfo>,
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds=[CROWDSALE_SEED, payer.key().as_ref()],
        bump    
    )]
    pub crowdsale_account: Account<'info, CrowdSale>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}


#[account]
pub struct CrowdSale {
    authority: Pubkey,
    min_price: u64,
    max_price: u64,
    start: u64,
    end: u64,
    price: u64,
    supply: u64,
    sold_amount: u64,
}

#[account]
pub struct UserInfo {
    bought_amount: u64,
    sol_amount: u64,
}

// helper function

pub fn calculate_reward_amount(crowdsale_account: &CrowdSale, amount: u64) -> Result<u64> {
    let sol_per_token = LAMPORTS_PER_SOL
        .checked_div(crowdsale_account.price)
        .unwrap();
    Ok(amount.checked_mul(sol_per_token).unwrap())
}

pub fn is_sale_active(crowdsale_account: &CrowdSale, clock: Clock) -> Result<bool> {
    let current_time = clock.unix_timestamp as u64;
    Ok(crowdsale_account.start <= current_time && current_time <= crowdsale_account.end)
}

pub fn is_price_valid(crowdsale_account: &CrowdSale, amount: u64) -> bool {
    crowdsale_account.min_price <= amount && amount <= crowdsale_account.max_price
}
