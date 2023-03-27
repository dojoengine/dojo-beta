use traits::TryInto;

impl U256TryIntoFelt252 of TryInto::<u256, felt252> {
    fn try_into(self: u256) -> Option<felt252> {
        let low: felt252 = self.low.try_into();
        let high: felt252 = self.low.try_into();
        // TODO: bounds checking
        low + high
    }
}

#[contract]
mod ERC20 {
    use dojo::world;
    use zeroable::Zeroable;
    use starknet::get_caller_address;
    use starknet::contract_address_const;
    use starknet::ContractAddress;
    use starknet::ContractAddressZeroable;

    struct Storage {
        world_address: ContractAddress,
        name: felt252,
        symbol: felt252,
        decimals: u8,
        total_supply: u256,
    }

    #[event]
    fn Transfer(from: ContractAddress, to: ContractAddress, value: u256) {}

    #[event]
    fn Approval(owner: ContractAddress, spender: ContractAddress, value: u256) {}

    #[constructor]
    fn constructor(
        world_address_: ContractAddress,
        name_: felt252,
        symbol_: felt252,
        decimals_: u8,
        initial_supply: u256,
        recipient: ContractAddress
    ) {
        world_address::write(world_address_);
        name::write(name_);
        symbol::write(symbol_);
        decimals::write(decimals_);
        assert(!recipient.is_zero(), 'ERC20: mint to the 0 address');
        total_supply::write(initial_supply);

        // balances::write(recipient, initial_supply);

        Transfer(contract_address_const::<0>(), recipient, initial_supply);
    }

    #[view]
    fn get_name() -> felt252 {
        name::read()
    }

    #[view]
    fn get_symbol() -> felt252 {
        symbol::read()
    }

    #[view]
    fn get_decimals() -> u8 {
        decimals::read()
    }

    #[view]
    fn get_total_supply() -> u256 {
        total_supply::read()
    }

    #[view]
    fn balance_of(account: ContractAddress) -> u256 {
        balances::read(account)
    }

    #[view]
    fn allowance(owner: ContractAddress, spender: ContractAddress) -> u256 {
        allowances::read((owner, spender))
    }

    #[external]
    fn transfer(spender: ContractAddress, recipient: ContractAddress, amount: u256) {
        ERC20_Transfer.execute(symbol,spender, recipient, amount);

        let calldata = ArrayTrait::<felt252>::new();
        calldata.append(starknet::get_contract_address().into());
        calldata.append(spender.into());
        calldata.append(recipient.into());
        calldata.append(amount.try_into());

        IWorldDispatcher { contract_address: world_address::read() }.execute('ERC20_TransferFrom', calldata.span());
       // TODO: update spend allowance

        Transfer(sender, recipient, amount);
    }

    //approval system
    #[external]
    fn approve(spender: ContractAddress, amount: u256) {
       ERC20_Approve.execute(symbol, spender, amount);
       Approval(get_caller_address(),spender,amount);
    }
   

    //approval system
    fn spend_allowance(owner: ContractAddress, spender: ContractAddress, amount: u256) {
        let current_allowance = allowances::read((owner, spender));
        let ONES_MASK = 0xffffffffffffffffffffffffffffffff_u128;
        let is_unlimited_allowance =
            current_allowance.low == ONES_MASK & current_allowance.high == ONES_MASK;
        if !is_unlimited_allowance {
            approve_helper(owner, spender, current_allowance - amount);
        }
    }
}
