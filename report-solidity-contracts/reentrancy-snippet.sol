function withdrawOneWei() public {
    msg.sender.call.value(1);
    balances[msg.sender] = balances[msg.sender] - 1;
}