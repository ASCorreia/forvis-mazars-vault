use anchor_lang::{
    prelude::*, 
    system_program::{
        Transfer, transfer
    }
};

declare_id!("2ZCGqymW3fhdgwJhP6oaubyg1ymgUvogxsHk3653pXhg");

#[program]
pub mod forvis_mazars_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        seeds = [b"state", user.key().as_ref()], 
        bump,
        space = VaultState::DISCRIMINATOR.len() + VaultState::INIT_SPACE,
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        init,
        payer = user,
        seeds = [b"records", user.key().as_ref()],
        bump,
        space = UserRecords::DISCRIMINATOR.len() + UserRecords::INIT_SPACE,
    )]
    pub user_records: Account<'info, UserRecords>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        // Get the amount of lamports needed to make the vault rent exempt
        let rent_exempt = Rent::get()?.minimum_balance(self.vault.to_account_info().data_len());

        // Transfer the rent-exempt amount from the user to the vault
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, rent_exempt)?;

        self.vault_state.vault_bump = bumps.vault;
        self.vault_state.state_bump = bumps.vault_state;
        self.vault_state.records_bump = bumps.user_records;

        self.user_records.set_inner(UserRecords {
            last_deposited: Record {
                amount: 0,
                timestamp: 0,
            },
            last_withdrawn: Record {
                amount: 0,
                timestamp: 0,
            },
        });

        Ok(())
    }  
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()], 
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        mut,
        seeds = [b"records", user.key().as_ref()],
        bump = vault_state.records_bump,
    )]
    pub user_records: Account<'info, UserRecords>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {

        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, amount)?;

        self.user_records.last_deposited = Record {
            amount,
            timestamp: Clock::get()?.unix_timestamp,
        };

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        mut,
        seeds = [b"records", user.key().as_ref()],
        bump = vault_state.records_bump,
    )]
    pub user_records: Account<'info, UserRecords>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        // Seeds must be defined with type &[&[&[_]]] so the compiler understands this type must be inferred
        // With this we can avoid creating a extra variable for not having the right lifetime
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault",
            // The AccountInfo struct lives long enough, as long as the transaction, 
            // so we can avoid creating an extra variabe by using it
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_ctx, amount)?;

        self.user_records.last_withdrawn = Record {
            amount,
            timestamp: Clock::get()?.unix_timestamp,
        };

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump,
        close = user,
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        mut,
        seeds = [b"records", user.key().as_ref()],
        bump = vault_state.records_bump,
        close = user,
    )]
    pub user_records: Account<'info, UserRecords>,
    pub system_program: Program<'info, System>,
}

impl<'info> Close<'info> {
    pub fn close(&mut self) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_ctx, self.vault.lamports())?;

        Ok(())
    }
}

#[derive(InitSpace)]
#[account]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
    pub records_bump: u8,
}

#[derive(InitSpace)]
#[account]
pub struct UserRecords {
    pub last_deposited: Record,
    pub last_withdrawn: Record,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, InitSpace)]
pub struct Record {
    pub amount: u64,
    pub timestamp: i64,
}