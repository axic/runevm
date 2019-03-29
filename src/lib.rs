extern crate ethereum_types;
extern crate evm;
extern crate ewasm_api;
extern crate parity_bytes as bytes;
extern crate vm;

use std::cmp;
use std::ops::Deref;
use std::sync::Arc;

use self::ethereum_types::{Address, H160, H256, U128, U256};

use self::bytes::Bytes;

use self::evm::Ext;
use self::evm::Factory;

use self::vm::{
    ActionParams, ActionValue, CallType, CleanDustMode, ContractCreateResult,
    CreateContractAddress, EnvInfo, GasLeft, MessageCallResult, Result, ReturnData, Schedule,
    TrapKind,
};

// For some explanation see ethcore/vm/src/tests.rs::FakeExt

#[derive(Default)]
struct EwasmExt {
    pub info: EnvInfo,
    pub schedule: Schedule,
    pub selfdestruct_address: Option<Address>,
}

impl vm::Ext for EwasmExt {
    /// Returns the storage value for a given key if reversion happens on the current transaction.
    fn initial_storage_at(&self, key: &H256) -> Result<H256> {
        unimplemented!()
    }

    /// Returns a value for given key.
    fn storage_at(&self, key: &H256) -> Result<H256> {
        // FIXME: why isn't there a From trait for converting between [u8;32] and H256?
        let key = key.0;
        let ret = ewasm_api::storage_load(&key);
        Ok(H256::from(ret))
    }

    /// Stores a value for given key.
    fn set_storage(&mut self, key: H256, value: H256) -> Result<()> {
        let key = key.0;
        let value = value.0;
        ewasm_api::storage_store(&key, &value);
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
        Ok(U256::from(U128::from(ewasm_api::external_balance(&origin))))
    }

    /// Returns address balance.
    fn balance(&self, address: &Address) -> Result<U256> {
        // FIXME: this type should just implement the From trait for the underlying type
        let address: [u8; 20] = address.0;
        Ok(U256::from(U128::from(ewasm_api::external_balance(
            &address,
        ))))
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
        // FIXME: set this properly
        //let gas_limit = u64::from(gas);
        let gas_limit = gas.as_u64();

        // FIXME: might not be good enough
        let gas_start = ewasm_api::gas_left();

        // FIXME: this type should just implement the From trait for the underlying type
        let receive_address: [u8; 20] = receive_address.0;

        let call_result = match call_type {
            CallType::Call => ewasm_api::call_mutable(
                gas_limit,
                &receive_address,
                &U128::from(value.unwrap_or_default()).into(),
                &data,
            ),
            CallType::CallCode => ewasm_api::call_code(
                gas_limit,
                &receive_address,
                &U128::from(value.unwrap_or_default()).into(),
                &data,
            ),
            CallType::DelegateCall => ewasm_api::call_delegate(gas_limit, &receive_address, &data),
            CallType::StaticCall => ewasm_api::call_static(gas_limit, &receive_address, &data),
            _ => panic!(),
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
        }

        // FIXME: no way to know if it ran out of gas? Handle it properly.
    }

    /// Returns code at given address
    fn extcode(&self, address: &Address) -> Result<Option<Arc<Bytes>>> {
        // FIXME: implement
        unimplemented!()
    }

    /// Returns code hash at given address
    fn extcodehash(&self, address: &Address) -> Result<Option<H256>> {
        // NOTE: only used by constantinople's EXTCODEHASH
        // FIXME: implement
        unimplemented!()
    }

    /// Returns code size at given address
    fn extcodesize(&self, address: &Address) -> Result<Option<usize>> {
        // FIXME: implement
        unimplemented!()
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

    // FIXME: do we need to set this?
    // params.call_type = if code.is_none() { CallType::Call } else { CallType::None };
    params.code_address = Address::from(ewasm_api::current_address());
    params.code = Some(Arc::new(ewasm_api::code_acquire()));
    params.address = params.code_address;
    params.sender = Address::from(ewasm_api::caller());
    params.origin = Address::from(ewasm_api::tx_origin());
    params.gas_price = U256::from(U128::from(ewasm_api::tx_gas_price()));
    params.gas = U256::from(startgas);
    params.data = Some(ewasm_api::calldata_acquire());

    let mut ext = EwasmExt::default();

    let mut instance = Factory::default().create(params, ext.schedule(), ext.depth());
    let result = instance.exec(&mut ext);
    // Could run `result.finalize(ext)` here, but processing manually seemed simpler.
    match result {
        Ok(Ok(GasLeft::Known(gas_left))) => {
            if ext.selfdestruct_address.is_some() {
                let beneficiary: [u8; 20] = ext.selfdestruct_address.unwrap().into();
                ewasm_api::selfdestruct(&beneficiary)
            } else {
                ewasm_api::finish()
            }
        }
        Ok(Ok(GasLeft::NeedsReturn {
            gas_left,
            data,
            apply_state,
        })) => ewasm_api::finish_data(&data.deref()),
        // FIXME: not sure what this state means
        Ok(Err(err)) => ewasm_api::revert(),
        // FIXME: add support for pushing the error message as revert data
        Err(err) => ewasm_api::revert(),
    }
}
