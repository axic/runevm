extern crate ewasm_api;
extern crate vm;
extern crate evm;
extern crate ethereum_types;
extern crate parity_bytes as bytes;

use std::sync::Arc;
use std::ops::Deref;
use std::cmp;

use self::ethereum_types::{U256, H256, H160, Address};

use self::bytes::Bytes;

use self::evm::Factory;

use self::vm::{
    EnvInfo, CreateContractAddress, ReturnData, CleanDustMode, ActionParams,
    ActionValue, Schedule, ContractCreateResult, MessageCallResult, CallType,
    Result, GasLeft
};

trait LEVec {
    fn to_le_vec(&self) -> Vec<u8>;
}

impl LEVec for U256 {
    fn to_le_vec(&self) -> Vec<u8> {
        let mut ret = [0u8; 32];
        self.to_little_endian(&mut ret);
        ret.to_vec()
    }
}

// For some explanation see ethcore/vm/src/tests.rs::FakeExt

#[derive(Default)]
struct EwasmExt {
    pub info: EnvInfo,
    pub schedule: Schedule,
    pub selfdestruct_address: Option<Address>
}

impl vm::Ext for EwasmExt {
    /// Returns a value for given key.
    fn storage_at(&self, key: &H256) -> Result<H256> {
        let ret = ewasm_api::storage_load(&key.to_vec());
        Ok(H256::from(ret.as_slice()))
    }

    /// Stores a value for given key.
    fn set_storage(&mut self, key: H256, value: H256) -> Result<()> {
        ewasm_api::storage_store(&key.to_vec(), &value.to_vec());
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
        Ok(U256::from(ewasm_api::external_balance(&origin).as_slice()))
    }

    /// Returns address balance.
    fn balance(&self, address: &Address) -> Result<U256> {
        Ok(U256::from(ewasm_api::external_balance(&address.to_vec()).as_slice()))
    }

    /// Returns the hash of one of the 256 most recent complete blocks.
    fn blockhash(&mut self, number: &U256) -> H256 {
        // FIXME: implement
        unimplemented!()
    }

    /// Creates new contract.
    ///
    /// Returns gas_left and contract address if contract creation was succesfull.
    fn create(&mut self, gas: &U256, value: &U256, code: &[u8], address: CreateContractAddress) -> ContractCreateResult {
        // FIXME: implement
        unimplemented!()
        // ContractCreateResult::Failed
    }

    /// Message call.
    ///
    /// Returns Err, if we run out of gas.
    /// Otherwise returns call_result which contains gas left
    /// and true if subcall was successfull.
    fn call(&mut self,
	gas: &U256,
	sender_address: &Address,
	receive_address: &Address,
	value: Option<U256>,
	data: &[u8],
	code_address: &Address,
	output: &mut [u8],
	call_type: CallType
    ) -> MessageCallResult {
        // FIXME: set this properly
        //let gas_limit = u64::from(gas);
        let gas_limit = gas.as_u64();

        // FIXME: might not be good enough
        let gas_start = ewasm_api::gas_left();

        let call_result = match call_type {
            CallType::Call => ewasm_api::call_mutable(gas_limit, &receive_address.to_vec(), &value.unwrap_or_default().to_le_vec(), &data.to_vec()),
            CallType::CallCode => ewasm_api::call_code(gas_limit, &receive_address.to_vec(), &value.unwrap_or_default().to_le_vec(), &data.to_vec()),
            CallType::DelegateCall => ewasm_api::call_delegate(gas_limit, &receive_address.to_vec(), &data.to_vec()),
            CallType::StaticCall => ewasm_api::call_static(gas_limit, &receive_address.to_vec(), &data.to_vec()),
            _ => panic!()
        };

        // FIXME: might not be good enough
        let gas_used = U256::from(ewasm_api::gas_left() - gas_start);

        match call_result {
            ewasm_api::CallResult::Successful => {
                // Retrieve the entire returndata as it needs to be returned
                let ret_len = ewasm_api::returndata_size();
                let ret = ewasm_api::returndata_copy(0, ret_len);

                // Copy from returndata into the requested output len
                // The requested len may be smaller than available or returndata may be smaller than requested
                let copy_len = cmp::min(output.len(), ret_len);
                output.copy_from_slice(&ret[0..copy_len]);

                MessageCallResult::Success(gas_used, ReturnData::new(ret, 0, ret_len))
            },
            ewasm_api::CallResult::Failure => MessageCallResult::Failed,
            ewasm_api::CallResult::Revert => {
                // Retrieve the entire returndata as it needs to be returned
                let ret_len = ewasm_api::returndata_size();
                let ret = ewasm_api::returndata_copy(0, ret_len);

                // Copy from returndata into the requested output len
                // The requested len may be smaller than available or returndata may be smaller than requested
                let copy_len = cmp::min(output.len(), ret_len);
                output.copy_from_slice(&ret[0..copy_len]);

                MessageCallResult::Reverted(gas_used, ReturnData::new(ret, 0, ret_len))
            },
            _ => panic!()
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

    /// Increments sstore refunds count by 1.
    fn inc_sstore_clears(&mut self) {
        // NOTE: used for gas refund on SSTORE deletion (non-zero to zero)
        // FIXME: implement
    }

    /// Decide if any more operations should be traced. Passthrough for the VM trace.
    fn trace_next_instruction(&mut self, _pc: usize, _instruction: u8, _current_gas: U256) -> bool { false }

    /// Prepare to trace an operation. Passthrough for the VM trace.
    fn trace_prepare_execute(&mut self, _pc: usize, _instruction: u8, _gas_cost: U256) {}

    /// Trace the finalised execution of a single instruction.
    fn trace_executed(&mut self, _gas_used: U256, _stack_push: &[U256], _mem_diff: Option<(usize, &[u8])>, _store_diff: Option<(U256, U256)>) {}

    /// Check if running in static context.
    fn is_static(&self) -> bool {
        // NOTE: this is used by CREATE/CALL*, but since ewasm in the upper layer will handle this anyway, we can just ignore it here
        false
    }
}

#[no_mangle]
pub extern fn main() {
    // It is fine using U256::zero() here because the main point of the
    // factory is to determine if gas is 64bit or not. In ewasm it is always 64bit.
    let mut instance = Factory::default().create(&U256::zero());

    let mut params = ActionParams::default();
    // FIXME: do we need to set this?
    // params.call_type = if code.is_none() { CallType::Call } else { CallType::None };
    params.code_address = Address::from(ewasm_api::current_address().as_slice());
    params.code = Some(Arc::new(ewasm_api::code_copy(0, ewasm_api::code_size())));
    params.address = params.code_address;
    params.sender = Address::from(ewasm_api::caller().as_slice());
    params.origin = Address::from(ewasm_api::tx_origin().as_slice());
    params.gas_price = U256::from(ewasm_api::tx_gas_price().as_slice());
    // NOTE: there is no tx_gas_limit in the EEI
    params.gas = U256::from(ewasm_api::gas_left());
    params.data = Some(ewasm_api::calldata_copy(0, ewasm_api::calldata_size()));

    let mut ext = EwasmExt::default();
    let result = instance.exec(params, &mut ext);
    // Could run `result.finalize(ext)` here, but processing manually seemed simpler.
    match result {
        Ok(GasLeft::Known(gas_left)) => {
            if ext.selfdestruct_address.is_some() {
                let beneficiary = ext.selfdestruct_address.unwrap().to_vec();
                ewasm_api::selfdestruct(&beneficiary)
            } else {
                ewasm_api::finish()
            }
        },
        Ok(GasLeft::NeedsReturn {gas_left, data, apply_state}) => ewasm_api::finish_data(&data.deref().to_vec()),
        // FIXME: add support for pushing the error message as revert data
        Err(err) => ewasm_api::revert()
    }
}
