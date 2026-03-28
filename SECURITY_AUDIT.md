# Security Audit Report: Bonfida Token Vesting Program

**Repository:** https://github.com/Bonfida/token-vesting
**Commit:** HEAD of main branch (shallow clone, 2026-03-28)
**Program ID:** `VestingbGKPFXCWuBvfkegQfZyiNwAJb9Ss623VQ5DA`
**Auditor:** Automated analysis
**Date:** 2026-03-28

---

## Executive Summary

The Bonfida Token Vesting program is a native (non-Anchor) Solana program that enables creation of token vesting schedules with time-locked releases. The program manages vesting contracts where tokens are locked and released to a destination address according to predefined schedules.

This audit identified **4 critical/high-severity** vulnerabilities and **3 medium-severity** issues that could lead to fund loss, unauthorized fund redirection, or denial of service.

---

## CRITICAL / HIGH SEVERITY FINDINGS

### V-01: Missing Owner Check on Vesting Account in `process_unlock` [CRITICAL]

**File:** `program/src/processor.rs`, lines 185-268

**Description:**
The `process_unlock` function validates that the `vesting_account` key matches the PDA derived from the provided seeds (line 198-202), but it **never checks that `vesting_account.owner == program_id`**. Compare this to `process_create` (line 99-101), which does check `*vesting_account.owner != *program_id`.

Without an owner check, an attacker can provide a fake account with the same public key derivation but owned by the System Program (or another program) that contains crafted data. While PDA derivation makes this harder to exploit directly (since the PDA is deterministic), if the vesting account were ever closed and its lamports drained (returning it to System Program ownership), an attacker could re-create the account with the same address but different data, potentially manipulating the vesting schedule to unlock funds prematurely or redirect them.

**Impact:** An attacker who can influence the account state (e.g., after account closure) could unlock tokens to an attacker-controlled destination by crafting fake vesting schedule data.

**Proof of concept:**
1. Wait for or cause a vesting account to be closed (lamports zeroed)
2. The account is now owned by the System Program
3. Re-create the account at the same PDA address with crafted data containing `destination_address` pointing to attacker's token account and `release_time` of 0
4. Call `process_unlock` -- the owner check is missing, so the crafted data is accepted
5. Tokens transfer from the vesting token account to the attacker's account

**Proposed fix:**
```rust
// Add after line 202 in process_unlock:
if *vesting_account.owner != *program_id {
    msg!("Program should own vesting account");
    return Err(ProgramError::InvalidArgument);
}
```

---

### V-02: Missing Owner Check on Vesting Account in `process_change_destination` [CRITICAL]

**File:** `program/src/processor.rs`, lines 270-318

**Description:**
The `process_change_destination` function never checks that `vesting_account.owner == program_id`. An attacker can supply a fake vesting account (not owned by the program) that contains crafted header data with a known `destination_address`. The attacker then provides proof of ownership of that destination address (which they control) and redirects the vesting contract's destination to their own token account.

Combined with calling `unlock` later, this allows full theft of vested tokens.

**Impact:** Complete theft of vested tokens by redirecting the destination address.

**Proof of concept:**
1. Attacker creates a fake account (same PDA key if possible, or exploits account closure) with `destination_address` set to an attacker-controlled token account
2. Attacker calls `change_destination` providing:
   - The fake vesting account (no owner check!)
   - Their token account as `destination_token_account`
   - Themselves as signer/owner
   - A new destination of their choice
3. The vesting contract's destination is now attacker-controlled
4. When tokens unlock, they go to the attacker

**Proposed fix:**
```rust
// Add at the beginning of process_change_destination, after getting vesting_account:
if *vesting_account.owner != *program_id {
    msg!("Program should own vesting account");
    return Err(ProgramError::InvalidArgument);
}
```

---

### V-03: Missing Mint Address Validation in `process_unlock` [HIGH]

**File:** `program/src/processor.rs`, lines 185-268

**Description:**
The `process_create` function stores a `mint_address` in the vesting schedule header (line 132), but `process_unlock` **never validates that the vesting token account's mint matches the stored `mint_address`**. The header's `mint_address` field is written during creation but never read or checked during unlock.

This means if an attacker can create a vesting token account with a different (worthless) mint that is still "owned" by the vesting PDA, they could potentially set up a scenario where unlocking sends the wrong tokens, or the validation fails silently.

More critically, since the `destination_token_account` is also not validated against the mint, tokens could be sent to a mismatched token account, which would cause the SPL Token transfer to fail -- but this also means the mint field serves no actual purpose as a security check.

**Impact:** The stored mint address provides a false sense of security. No validation ensures that the token account used during unlock matches the intended mint, potentially enabling token substitution attacks in edge cases.

**Proposed fix:**
```rust
// In process_unlock, after unpacking vesting_token_account_data:
if vesting_token_account_data.mint != header_state.mint_address {
    msg!("Vesting token account mint does not match the expected mint");
    return Err(ProgramError::InvalidArgument);
}

// Also validate destination token account mint:
let destination_token_account_data = Account::unpack(&destination_token_account.data.borrow())?;
if destination_token_account_data.mint != header_state.mint_address {
    msg!("Destination token account mint does not match");
    return Err(ProgramError::InvalidArgument);
}
```

---

### V-04: Integer Overflow in `process_unlock` Total Amount Calculation [HIGH]

**File:** `program/src/processor.rs`, line 232

**Description:**
In `process_unlock`, the total amount to transfer is calculated with unchecked addition:

```rust
total_amount_to_transfer += s.amount;
```

While `process_create` correctly uses `checked_add` (line 151), the unlock path does NOT. If a vesting contract has schedules whose amounts sum to more than `u64::MAX`, the `total_amount_to_transfer` will silently overflow/wrap around.

In practice, Rust in release mode (which Solana BPF programs use) wraps on overflow by default (unlike debug mode which panics). This could lead to transferring fewer tokens than expected from the vesting account, leaving tokens permanently locked. Or worse, if the wrapping results in a very small number, the attacker who controls the destination could extract a small amount while the rest remains unclaimable.

Note: While `process_create` validates the sum of schedule amounts with `checked_add`, an attacker could potentially craft a scenario through account data manipulation (see V-01, V-02) where the schedule amounts in the account exceed what was originally validated.

**Impact:** Potential for tokens to be permanently locked or incorrect amounts to be unlocked due to integer overflow.

**Proposed fix:**
```rust
// Replace line 232 with:
total_amount_to_transfer = total_amount_to_transfer
    .checked_add(s.amount)
    .ok_or(ProgramError::InvalidAccountData)?;
```

---

## MEDIUM SEVERITY FINDINGS

### V-05: Missing SPL Token Program Validation in `process_create` [MEDIUM]

**File:** `program/src/processor.rs`, line 82

**Description:**
In `process_create`, the `spl_token_account` (first account) is used to invoke the SPL Token transfer, but its program ID is **never validated** against `spl_token::id()`. Compare this to `process_unlock` (line 204-207), which DOES check `spl_token_account.key != &spl_token::id()`.

This means an attacker could pass a malicious program as the "SPL Token program" during the Create instruction. The CPI call to transfer tokens could invoke a malicious program instead of the real SPL Token program. The malicious program could return `Ok(())` without actually transferring any tokens, meaning the vesting account's state would be marked as initialized with schedules, but the vesting token account would contain zero tokens.

The vesting contract would then be in an invalid state: it records schedules for tokens that were never actually deposited.

**Impact:** Vesting contracts could be created without actual token deposits, creating phantom vesting schedules. While this primarily harms the creator (their tokens stay in their own account), it breaks the invariant that vesting contracts are funded, and could be used for social engineering (showing someone a "funded" vesting contract that contains no tokens).

**Proposed fix:**
```rust
// Add at the beginning of process_create:
if *spl_token_account.key != spl_token::id() {
    msg!("The provided spl token program account is invalid");
    return Err(ProgramError::InvalidArgument);
}
```

---

### V-06: No Validation That `new_destination_token_account` is a Valid Token Account [MEDIUM]

**File:** `program/src/processor.rs`, lines 270-318

**Description:**
In `process_change_destination`, the `new_destination_token_account` is accepted without any validation. It is not checked to be:
- A valid SPL Token account
- An account with the correct mint matching the vesting contract's mint
- An initialized token account at all

The destination address is simply stored as a pubkey. If someone changes the destination to an address that is not a valid token account (or a token account with the wrong mint), subsequent `unlock` calls will fail, effectively **locking the tokens permanently**.

This could be exploited by a malicious destination owner who wants to grief a vesting contract (e.g., an employee who was fired might set the destination to an invalid account to prevent the company from recovering tokens after the vesting period).

**Impact:** Tokens can be permanently locked by changing the destination to an invalid or wrong-mint token account.

**Proposed fix:**
```rust
// Validate new_destination_token_account in process_change_destination:
let new_dest_data = Account::unpack(&new_destination_token_account.data.borrow())?;
let state = VestingScheduleHeader::unpack(
    &vesting_account.data.borrow()[..VestingScheduleHeader::LEN],
)?;
if new_dest_data.mint != state.mint_address {
    msg!("New destination token account mint does not match vesting contract mint");
    return Err(ProgramError::InvalidArgument);
}
```

---

### V-07: `process_init` Does Not Validate Payer is Signer [MEDIUM]

**File:** `program/src/processor.rs`, lines 28-70

**Description:**
The `process_init` function takes a `payer` account and uses it to fund the creation of the vesting account via `create_account`. However, it never explicitly checks `payer.is_signer`. While the System Program's `create_account` instruction will itself require the payer to be a signer (so this will fail at the CPI level), the explicit check is a defense-in-depth best practice. More importantly, the lack of an explicit check means the error message on failure will be a generic System Program error rather than a clear "payer must be signer" message from the vesting program.

Additionally, `process_init` does not validate that `system_program_account.key == &system_program::id()`, which means a fake system program could be passed. Similar to V-05, a malicious "system program" could pretend to create the account without actually doing so.

**Impact:** Lower severity since the System Program CPI provides an implicit check, but the missing system program ID validation could allow initialization with a fake system program.

**Proposed fix:**
```rust
// Add explicit checks in process_init:
if !payer.is_signer {
    msg!("Payer must be a signer");
    return Err(ProgramError::MissingRequiredSignature);
}
if *system_program_account.key != system_program::id() {
    msg!("Invalid system program");
    return Err(ProgramError::InvalidArgument);
}
```

---

## INFORMATIONAL / LOW SEVERITY FINDINGS

### V-08: Seeds Provided in Instruction Data Enable Attacker-Chosen PDA [LOW]

**Description:**
The seed used for PDA derivation is passed as instruction data (a 32-byte array) rather than being derived from meaningful parameters (e.g., creator pubkey, destination, mint). This means:
- Anyone can create a vesting contract with any seed
- There is no enforced relationship between the seed and the parties involved
- Seeds are effectively opaque, making it harder to discover/verify vesting contracts on-chain

This is a design choice rather than a bug, but it means the program relies entirely on off-chain coordination to track which seeds correspond to which vesting contracts.

### V-09: No Account Close / Reclaim Mechanism [LOW]

**Description:**
Once a vesting contract is fully unlocked (all schedule amounts set to 0), the vesting account remains allocated on-chain with its rent-exempt lamports locked. There is no instruction to close the vesting account and reclaim the rent. Over time, this leads to permanent locking of SOL in empty vesting accounts.

### V-10: Clock Sysvar Deprecation [INFO]

**Description:**
The program uses `Clock::from_account_info(&clock_sysvar_account)` (line 226). While this still works, Solana has deprecated passing the clock sysvar as an account in favor of `Clock::get()` which reads it directly via syscall, saving an account in the transaction.

---

## Summary Table

| ID | Severity | Title | Status |
|----|----------|-------|--------|
| V-01 | CRITICAL | Missing owner check in `process_unlock` | Open |
| V-02 | CRITICAL | Missing owner check in `process_change_destination` | Open |
| V-03 | HIGH | Missing mint validation in `process_unlock` | Open |
| V-04 | HIGH | Integer overflow in `process_unlock` | Open |
| V-05 | MEDIUM | Missing SPL Token program validation in `process_create` | Open |
| V-06 | MEDIUM | No validation of new destination token account | Open |
| V-07 | MEDIUM | Missing payer signer check and system program validation in `process_init` | Open |
| V-08 | LOW | Attacker-chosen PDA seeds | Open |
| V-09 | LOW | No account close mechanism | Open |
| V-10 | INFO | Deprecated clock sysvar usage | Open |

---

## Notes on Exploitability

The critical vulnerabilities (V-01, V-02) have a key prerequisite: the attacker needs the vesting account PDA to be in a state where it is no longer owned by the program. Under normal operation, once a vesting account is created via `process_init`, it is owned by the program and cannot be easily manipulated. However:

1. **Account closure race:** If the Solana runtime garbage-collects a zero-lamport account (which can happen), the account could revert to System Program ownership, enabling V-01/V-02.
2. **Future program upgrades:** If the program is ever upgraded to add an account-close instruction, V-01/V-02 become directly exploitable.
3. **Defense in depth:** Even if current exploitation is difficult, the missing owner checks are clear violations of Solana security best practices and should be fixed.

The HIGH findings (V-03, V-04) are more directly exploitable and represent clear code defects.

---

## Methodology

- Manual source code review of all Rust source files in `program/src/`
- Cross-referencing instruction handlers with account validation patterns
- Checking for common Solana vulnerability patterns (missing signer checks, missing owner checks, arbitrary CPI, integer overflow, PDA issues, reinitialization, account confusion)
- Comparison of validation logic across different instruction handlers for inconsistencies
