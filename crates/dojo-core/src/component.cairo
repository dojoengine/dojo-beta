trait Component<T> {
    fn name(self: @T) -> felt252;
    fn keys(self: @T) -> Span<felt252>;
    fn pack(self: @T) -> Span<felt252>;
    fn unpack(ref unpacked: Array<felt252>, ref packed: Span<felt252>);
}

#[starknet::interface]
trait IComponent<T> {
    fn name(self: @T) -> felt252;
    fn layout(self: @T) -> Span<felt252>;
    fn schema(self: @T) -> Span<dojo::Member>;
}

#[starknet::interface]
trait ISystem<T> {
    fn name(self: @T) -> felt252;
}
