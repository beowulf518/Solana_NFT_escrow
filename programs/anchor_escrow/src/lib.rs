use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token::{self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;
use metaplex_token_metadata::state::Metadata;

declare_id!("25vAKs29xER5CYkxV58MVnN1FZmgj235zA7kHxd6RFk8");


#[account]
pub struct EscrowInfo {
    pub is_initialized: bool,
    pub seller: Pubkey,
    pub token_account_pubkey: Pubkey,
    pub mint_key: Pubkey,
    pub amount: u64,
    pub index: u8,
}

#[program]
#[warn(unused_parens)]
pub mod anchor_escrow {
    use super::*;

    const ESCROW_PDA_SEED: &[u8] = b"escrow";

    pub fn listing(
        ctx: Context<List>,
        _vault_account_bump: u8,
        selling_amount: u64,
        index: u8,
    ) -> ProgramResult{
        if !ctx.accounts.initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if selling_amount < (1000 as u64){
            return Err(ProgramError::InvalidInstructionData);
        }
        if selling_amount >= (1000000000 as u64){
            return Err(ProgramError::InvalidInstructionData);
        }
        if ctx.accounts.escrow_account.is_initialized{
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        if ctx.accounts.escrow_account.amount > 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        ctx.accounts.escrow_account.is_initialized = true;
        ctx.accounts.escrow_account.seller = *ctx.accounts.initializer.key;
        ctx.accounts.escrow_account.token_account_pubkey = *ctx.accounts.token_account.to_account_info().key;
        ctx.accounts.escrow_account.mint_key = *ctx.accounts.mint_key.to_account_info().key;
        ctx.accounts.escrow_account.amount = selling_amount;
        ctx.accounts.escrow_account.index = index;

        let escrow_key = ctx.accounts.escrow_account.key();

        let pda_seed = &[
            ESCROW_PDA_SEED,
            escrow_key.as_ref(),
        ];

        let (vault_authority, _vault_authority_bump) =
            Pubkey::find_program_address(pda_seed, ctx.program_id);
        
        msg!("owner {}", vault_authority);

        token::set_authority(
            ctx.accounts.set_authority_context(),
            AuthorityType::AccountOwner,
            Some(vault_authority),
        )?;

        //token::transfer(
        //    ctx.accounts.transfer_to_pda_context(),
        //    selling_amount,
        //)?;

        Ok(())

    }

    pub fn buy(
        ctx: Context<Buy>,
        _vault_account_bump: u8,
        expected_price: u64,
    ) -> ProgramResult{
        if !ctx.accounts.buyer.is_signer
        {
            return Err(ProgramError::MissingRequiredSignature)
        }
        if ctx.accounts.escrow_info.amount != expected_price as u64
        {
            return Err(ProgramError::InvalidAccountData)
        }
        if ctx.accounts.escrow_info.seller != ctx.accounts.initializers_main_account.key(){
            return Err(ProgramError::InvalidAccountData);
        }
        if ctx.accounts.escrow_info.mint_key != ctx.accounts.mint_key.key(){
            return Err(ProgramError::InvalidAccountData);
        }
        if ctx.accounts.buyer.key() == ctx.accounts.initializers_main_account.key(){
            return Err(ProgramError::InvalidAccountData);
        }
        
        let escrow_key = ctx.accounts.buyer.key();

        let buynow_pda_seeds = &[
            ESCROW_PDA_SEED,
            escrow_key.as_ref(),
        ];

        let (buynow_pda, nonce) =
            Pubkey::find_program_address(buynow_pda_seeds, ctx.program_id);
        

        if ctx.accounts.pda_account.key() != buynow_pda{
            return Err(ProgramError::InvalidAccountData);
        }

        const PREFIX: &str = "metadata";
        let key = ctx.accounts.token_meta_program.key();
        //let metadata_program_id = Pubkey::new(key);
        // seeds for metadata pda
        let metadata_seeds = &[
            PREFIX.as_bytes(),
            key.as_ref(),
            ctx.accounts.escrow_info.mint_key.as_ref(),
        ];
    
        let (metadata_key, _metadata_bump_seed) =
            Pubkey::find_program_address(metadata_seeds, &ctx.accounts.token_meta_program.key());
        
        //msg!("metadata_key {}", metadata_key);
        //msg!("metadata_info.key {}", ctx.accounts.metadata_info.key);
        //msg!("token_meta_program {}", ctx.accounts.token_meta_program.key());
        // validation check for correct accounts send from the client side
        if *ctx.accounts.metadata_info.key != metadata_key{
            return Err(ProgramError::InvalidAccountData);
        }

        //changes
        let size = ctx.accounts.escrow_info.amount;

        // unpack the metadata from the metadata pda
        //let metadata = Metadata::from_account_info(&ctx.accounts.metadata_info)?;

        // seller fee basis points from the metadata
        //let fees = metadata.data.seller_fee_basis_points;
        //let total_fee = ((fees as u64)*size)/10000;

        let mut remaining_fee = size;
        /*match metadata.data.creators {
            Some(creators) => {
                for creator in creators {
                    let pct = creator.share as u64;
                    let creator_fee = (pct*(total_fee))/100;
                    remaining_fee = remaining_fee - creator_fee;
                    let creator_acc_web = &ctx.accounts.creator_acc_web;
                    if *creator_acc_web.key != creator.address {
                        return Err(ProgramError::InvalidAccountData);
                    }

                    // sending royalties to the creators of the NFT
                    if creator_fee > 0 {

                        token::transfer(
                            ctx.accounts.transfer_to_pda_context(),
                            creator_fee,
                        )?;
                    }
                }
            }
            None => {
                msg!("No creators found in metadata");
            }
        }*/
        msg!("remaining fee {}", remaining_fee);
        //token::transfer(
        //    ctx.accounts.transfer_to_initializer(),
        //    remaining_fee,
        //)?;
        msg!("authority {}", ctx.accounts.token_account_authority.key());
        msg!("owner {}", ctx.accounts.pdas_token_account.owner);
        token::set_authority(
            ctx.accounts.set_authority_context(),
            AuthorityType::AccountOwner,
            Some(ctx.accounts.buyer.key()),
        )?;

        ctx.accounts.escrow_info.is_initialized = false;
        Ok(())
    }

    pub fn cancel(
        ctx: Context<Cancel>,
    ) -> ProgramResult{
        ctx.accounts.escrow_info.key();
        //if ctx.accounts.escrow_info.owner != ctx.program_id{
        //    return Err(ProgramError::IncorrectProgramId);
        //}
        if ctx.accounts.escrow_info.seller != ctx.accounts.user.key(){
            return Err(ProgramError::InvalidAccountData);
        }
        if ctx.accounts.escrow_info.token_account_pubkey != ctx.accounts.pdas_token_account.key(){
            return Err(ProgramError::InvalidAccountData);
        }

        let escrow = &mut ctx.accounts.escrow_info.key();

        //change get a pda for escrow program
        const PDA_PREFIX: &str = "escrow";
        let pda_seed = &[
            PDA_PREFIX.as_bytes(),
            (escrow).as_ref(),
        ];
        //pda
        let (pda, nonce) = Pubkey::find_program_address(pda_seed, ctx.program_id);

        let user = ctx.accounts.user.key();
        let new_pda_seed = &[
            PDA_PREFIX.as_bytes(),
            (user).as_ref(),
        ];
        //pda
        let (new_pda, nonce) = Pubkey::find_program_address(new_pda_seed, ctx.program_id);

        if ctx.accounts.pda_account.key() != pda{
            return Err(ProgramError::InvalidAccountData);
        }
        
        msg!("owner {}", ctx.accounts.pdas_token_account.owner);
        
        token::set_authority(
            ctx.accounts.set_authority_context(),
            AuthorityType::AccountOwner,
            Some(new_pda),
        )?;

        ctx.accounts.escrow_info.is_initialized = false;
        Ok(())
    }
}

// Utils (fully implemented)

impl<'info> List<'info> {
    fn set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.token_account.to_account_info().clone(),
            current_authority: self.initializer.clone(),
        };
        CpiContext::new(self.initializer.clone(), cpi_accounts)
    }
}

impl<'info> Buy<'info> {
    fn set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.pdas_token_account.to_account_info().clone(),
            current_authority: self.token_account_authority.clone(),
        };
        CpiContext::new(self.initializers_main_account.clone(), cpi_accounts)
    }

    fn transfer_to_pda_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.buyer.to_account_info().clone(),
            to: self.creator_acc_web.to_account_info().clone(),
            authority: self.buyer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn transfer_to_initializer(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.buyer.to_account_info().clone(),
            to: self.initializers_main_account.to_account_info().clone(),
            authority: self.buyer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

impl<'info> Cancel<'info> {
    fn set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.pdas_token_account.to_account_info().clone(),
            current_authority: self.pda_account.clone(),
        };
        CpiContext::new(self.user.clone(), cpi_accounts)
    }
}

// Instructions (fully implementated)

#[derive(Accounts)]
#[instruction(vault_account_bump: u8, initializer_amount: u64)]
pub struct List<'info> {
    #[account(mut, signer)]
    pub initializer: AccountInfo<'info>,
    pub mint_key: Account<'info, Mint>,
    #[account(
        init,
        seeds = [b"token-seed".as_ref(), &escrow_account.key().as_ref()],
        bump = vault_account_bump,
        payer = initializer,
        token::mint = mint_key,
        token::authority = initializer,
    )]
    pub token_account: Account<'info, TokenAccount>,
    #[account(zero)]
    pub escrow_account: Box<Account<'info, EscrowInfo>>,
    pub system_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut, signer)]
    pub buyer: AccountInfo<'info>,
    pub mint_key: Account<'info, Mint>,
    #[account(mut)]
    pub escrow_info: Box<Account<'info, EscrowInfo>>,
    #[account(mut)]
    pub initializers_main_account: AccountInfo<'info>,
    #[account(mut)]
    pub pdas_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pda_account: AccountInfo<'info>,
    #[account(mut)]
    pub metadata_info: AccountInfo<'info>,
    #[account(mut)]
    pub token_account_authority: AccountInfo<'info>,
    #[account(mut)]
    pub creator_acc_web: AccountInfo<'info>,
    #[account(executable)]
    pub token_program: AccountInfo<'info>,
    pub token_meta_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Cancel<'info>{
    #[account(mut, signer)]
    pub user: AccountInfo<'info>,
    #[account(mut)]
    pub pda_account: AccountInfo<'info>,
    #[account(mut)]
    pub pdas_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub escrow_info: Box<Account<'info, EscrowInfo>>,
    #[account(executable)]
    pub token_program: AccountInfo<'info>,
}

