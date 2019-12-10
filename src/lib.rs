extern crate ewasm_api;

use standalone_parity_evm::*;

use std::ops::Deref;
use std::sync::Arc;

use ewasm_api::types::{Bytes20, Bytes32, Uint128};

trait AsCheckedU64 {
    fn as_checked_u64(&self) -> u64;
}

impl AsCheckedU64 for U256 {
    fn as_checked_u64(&self) -> u64 {
        if self > &U256::from("ffffffffffffffff") {
            ewasm_api::abort();
        }
        self.as_u64()
    }
}

// For some explanation see ethcore/vm/src/tests.rs::FakeExt

#[derive(Default)]
struct EwasmExt {
    pub info: EnvInfo,
    pub schedule: Schedule,
    pub selfdestruct_address: Option<Address>,
}

impl Ext for EwasmExt {
    /// Returns the storage value for a given key if reversion happens on the current transaction.
    fn initial_storage_at(&self, key: &H256) -> Result<H256> {
        unimplemented!()
    }

    /// Returns a value for given key.
    fn storage_at(&self, key: &H256) -> Result<H256> {
        let ret: [u8; 32] = ewasm_api::storage_load(&Bytes32::from(key.0)).into();
        Ok(H256::from(ret))
    }

    /// Stores a value for given key.
    fn set_storage(&mut self, key: H256, value: H256) -> Result<()> {
        ewasm_api::storage_store(&Bytes32::from(key.0), &Bytes32::from(value.0));
        Ok(())
    }

    /// Determine whether an account exists.
    fn exists(&self, address: &Address) -> Result<bool> {
        // NOTE: used by SELFDESTRUCT/CALL for gas metering (not used here now since we don't charge gas)
        // FIXME: implement
        unimplemented!()
    }

    /// Determine whether an account exists and is not null (zero balance/nonce, no code).
    fn exists_and_not_null(&self, address: &Address) -> Result<bool> {
        // NOTE: used by SELFDESTRUCT/CALL for gas metering (not used here now since we don't charge gas)
        // FIXME: implement
        unimplemented!()
    }

    /// Balance of the origin account.
    fn origin_balance(&self) -> Result<U256> {
        // NOTE: used by SLEFDESTRUCT for gas metering (not used here now since we don't charge gas)
        let origin = ewasm_api::tx_origin();
        let balance: [u8; 16] = ewasm_api::external_balance(&origin).into();
        Ok(U256::from(U128::from_little_endian(&balance)))
    }

    /// Returns address balance.
    fn balance(&self, address: &Address) -> Result<U256> {
        let balance: [u8; 16] = ewasm_api::external_balance(&Bytes20::from(address.0)).into();
        Ok(U256::from(U128::from_little_endian(&balance)))
    }

    /// Returns the hash of one of the 256 most recent complete blocks.
    fn blockhash(&mut self, number: &U256) -> H256 {
        // FIXME: implement
        unimplemented!()
    }

    /// Creates new contract.
    ///
    /// Returns gas_left and contract address if contract creation was succesfull.
    fn create(
        &mut self,
        gas: &U256,
        value: &U256,
        code: &[u8],
        address: CreateContractAddress,
        trap: bool,
    ) -> ::std::result::Result<ContractCreateResult, TrapKind> {
        // FIXME: implement
        unimplemented!()
        // ContractCreateResult::Failed
    }

    /// Message call.
    ///
    /// Returns Err, if we run out of gas.
    /// Otherwise returns call_result which contains gas left
    /// and true if subcall was successfull.
    fn call(
        &mut self,
        gas: &U256,
        sender_address: &Address,
        receive_address: &Address,
        value: Option<U256>,
        data: &[u8],
        code_address: &Address,
        call_type: CallType,
        trap: bool,
    ) -> ::std::result::Result<MessageCallResult, TrapKind> {
        let gas_limit = gas.as_checked_u64();

        // FIXME: might not be good enough
        let gas_start = ewasm_api::gas_left();

        let value: [u8; 16] = U128::from(value.unwrap_or_default()).into();

        let call_result = match call_type {
            CallType::Call => ewasm_api::call_mutable(
                gas_limit,
                &Bytes20::from(receive_address.0),
                &Uint128::from(value),
                &data,
            ),
            CallType::CallCode => ewasm_api::call_code(
                gas_limit,
                &Bytes20::from(receive_address.0),
                &Uint128::from(value),
                &data,
            ),
            CallType::DelegateCall => {
                ewasm_api::call_delegate(gas_limit, &Bytes20::from(receive_address.0), &data)
            }
            CallType::StaticCall => {
                ewasm_api::call_static(gas_limit, &Bytes20::from(receive_address.0), &data)
            }
            _ => ewasm_api::abort(),
        };

        // FIXME: might not be good enough
        let gas_used = U256::from(ewasm_api::gas_left() - gas_start);

        match call_result {
            ewasm_api::CallResult::Successful => {
                // Retrieve the entire returndata as it needs to be returned
                let ret = ewasm_api::returndata_acquire();

                let ret_len = ret.len();
                Ok(MessageCallResult::Success(
                    gas_used,
                    ReturnData::new(ret, 0, ret_len),
                ))
            }
            ewasm_api::CallResult::Failure => Ok(MessageCallResult::Failed),
            ewasm_api::CallResult::Revert => {
                // Retrieve the entire returndata as it needs to be returned
                let ret = ewasm_api::returndata_acquire();

                let ret_len = ret.len();
                Ok(MessageCallResult::Reverted(
                    gas_used,
                    ReturnData::new(ret, 0, ret_len),
                ))
            }
            ewasm_api::CallResult::Unknown => ewasm_api::abort(),
        }

        // FIXME: no way to know if it ran out of gas? Handle it properly.
    }

    /// Returns code at given address
    fn extcode(&self, address: &Address) -> Result<Option<Arc<Bytes>>> {
        Ok(Some(Arc::new(ewasm_api::external_code_acquire(
            &Bytes20::from(address.0),
        ))))
    }

    /// Returns code hash at given address
    fn extcodehash(&self, address: &Address) -> Result<Option<H256>> {
        // NOTE: only used by constantinople's EXTCODEHASH
        // FIXME: implement
        unimplemented!()
    }

    /// Returns code size at given address
    fn extcodesize(&self, address: &Address) -> Result<Option<usize>> {
        Ok(Some(ewasm_api::external_code_size(&Bytes20::from(
            address.0,
        ))))
    }

    /// Creates log entry with given topics and data
    fn log(&mut self, topics: Vec<H256>, data: &[u8]) -> Result<()> {
        // FIXME: implement
        unimplemented!()
    }

    /// Should be called when transaction calls `RETURN` opcode.
    /// Returns gas_left if cost of returning the data is not too high.
    fn ret(self, gas: &U256, data: &ReturnData, apply_state: bool) -> Result<U256> {
        // NOTE: this is only called through finalize(), but we are not using it
        // so it should be safe to ignore it here
        unimplemented!()
    }

    /// Should be called when contract commits suicide.
    /// Address to which funds should be refunded.
    fn suicide(&mut self, refund_address: &Address) -> Result<()> {
        // NOTE: this will be handled after stopping execution with StopExecution
        self.selfdestruct_address = Some(*refund_address);
        Ok(())
    }

    /// Returns schedule.
    fn schedule(&self) -> &Schedule {
        &self.schedule
    }

    /// Returns environment info.
    fn env_info(&self) -> &EnvInfo {
        &self.info
    }

    /// Returns the chain ID of the blockchain
    fn chain_id(&self) -> u64 {
        unimplemented!()
    }

    /// Returns current depth of execution.
    ///
    /// If contract A calls contract B, and contract B calls C,
    /// then A depth is 0, B is 1, C is 2 and so on.
    fn depth(&self) -> usize {
        // FIXME: implement
        0
    }

    /// Increments sstore refunds counter.
    fn add_sstore_refund(&mut self, value: usize) {
        unimplemented!()
    }

    /// Decrements sstore refunds counter.
    fn sub_sstore_refund(&mut self, value: usize) {
        unimplemented!()
    }

    /// Decide if any more operations should be traced. Passthrough for the VM trace.
    fn trace_next_instruction(&mut self, _pc: usize, _instruction: u8, _current_gas: U256) -> bool {
        false
    }

    /// Prepare to trace an operation. Passthrough for the VM trace.
    fn trace_prepare_execute(
        &mut self,
        _pc: usize,
        _instruction: u8,
        _gas_cost: U256,
        _mem_written: Option<(usize, usize)>,
        _store_written: Option<(U256, U256)>,
    ) {
    }

    /// Trace the finalised execution of a single instruction.
    fn trace_executed(&mut self, _gas_used: U256, _stack_push: &[U256], _mem: &[u8]) {}

    /// Check if running in static context.
    fn is_static(&self) -> bool {
        // NOTE: this is used by CREATE/CALL*, but since ewasm in the upper layer will handle this anyway, we can just ignore it here
        false
    }
}

#[no_mangle]
pub extern "C" fn main() {
    // NOTE: There is no tx_gas_limit in the EEI. As a workaround query "gasLeft"
    //       as soon as possible.
    let startgas = ewasm_api::gas_left();

    let mut params = ActionParams::default();

    let current_address: [u8; 20] = ewasm_api::current_address().into();
    let caller: [u8; 20] = ewasm_api::caller().into();
    let tx_origin: [u8; 20] = ewasm_api::tx_origin().into();
    let tx_gas_price: [u8; 16] = ewasm_api::tx_gas_price().into();

    // FIXME: do we need to set this?
    // params.call_type = if code.is_none() { CallType::Call } else { CallType::None };
    params.code_address = Address::from(current_address);
    params.code = Some(Arc::new(ewasm_api::code_acquire()));
    params.address = params.code_address;
    params.sender = Address::from(caller);
    params.origin = Address::from(tx_origin);
    params.gas_price = U256::from(U128::from_little_endian(&tx_gas_price));
    // NOTE: there is no tx_gas_limit in the EEI
    params.gas = U256::from(startgas);
    params.data = Some(ewasm_api::calldata_acquire());

    let mut ext = EwasmExt::default();

    // TODO: should create a proper implementation for default() on EwasmExt which does this
    ext.schedule = Schedule::new_byzantium();

    let block_coinbase: [u8; 20] = ewasm_api::block_coinbase().into();
    let block_difficulty: [u8; 32] = ewasm_api::block_difficulty().into();

    // Set block environment information
    // TODO: do this via lazy loading
    ext.info.author = Address::from(block_coinbase);
    ext.info.difficulty = U256::from_little_endian(&block_difficulty);
    ext.info.number = ewasm_api::block_number();
    ext.info.timestamp = ewasm_api::block_timestamp();
    ext.info.gas_limit = U256::from(ewasm_api::block_gas_limit());

    let instance = Factory::default().create(params, ext.schedule(), ext.depth());
    let result = instance.exec(&mut ext);

    // Could run `result.finalize(ext)` here, but processing manually seemed simpler.
    match result {
        Ok(Ok(GasLeft::Known(gas_left))) => {
            ewasm_api::consume_gas(startgas - gas_left.as_checked_u64());
            if ext.selfdestruct_address.is_some() {
                let beneficiary: [u8; 20] = ext.selfdestruct_address.unwrap().into();
                ewasm_api::selfdestruct(&ewasm_api::types::Bytes20::from(beneficiary))
            } else {
                ewasm_api::finish()
            }
        }
        Ok(Ok(GasLeft::NeedsReturn {
            gas_left,
            data,
            apply_state,
        })) => {
            ewasm_api::consume_gas(startgas - gas_left.as_checked_u64());
            if apply_state {
                ewasm_api::finish_data(&data.deref())
            } else {
                ewasm_api::revert_data(&data.deref())
            }
        }
        // FIXME: not sure what this state means
        Ok(Err(_err)) => ewasm_api::abort(),
        // FIXME: add support for pushing the error message as revert data
        Err(_err) => ewasm_api::abort(),
    }
}
