pragma solidity ^0.4.24;

/**
 * @title ERC20 interface
 * @dev see https://github.com/ethereum/EIPs/issues/20
 * @dev copied from https://github.com/OpenZeppelin/openzeppelin-contracts
 */
interface IERC20 {
  /**
   * @dev Total number of tokens in existence
   * Selector: 18160ddd
   */
  function totalSupply() external view returns (uint256);

  /**
   * @dev Gets the balance of the specified address.
   * Selector: 70a08231
   * @param owner The address to query the balance of.
   * @return An uint256 representing the amount owned by the passed address.
   */
  function balanceOf(address who) external view returns (uint256);

  /**
   * @dev Function to check the amount of tokens that an owner allowed to a spender.
   * Selector: dd62ed3e
   * @param owner address The address which owns the funds.
   * @param spender address The address which will spend the funds.
   * @return A uint256 specifying the amount of tokens still available for the spender.
   */
  function allowance(address owner, address spender)
    external view returns (uint256);

  /**
   * @dev Transfer token for a specified address
   * Selector: a9059cbb
   * @param to The address to transfer to.
   * @param value The amount to be transferred.
   * @return true if the transfer was succesful, revert otherwise.
   */
  function transfer(address to, uint256 value) external returns (bool);

  /**
   * @dev Approve the passed address to spend the specified amount of tokens on behalf of msg.sender.
   * Beware that changing an allowance with this method brings the risk that someone may use both the old
   * and the new allowance by unfortunate transaction ordering. One possible solution to mitigate this
   * race condition is to first reduce the spender's allowance to 0 and set the desired value afterwards:
   * https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729
   * Selector: 095ea7b3
   * @param spender The address which will spend the funds.
   * @param value The amount of tokens to be spent.
   * @return true, this cannot fail
   */
  function approve(address spender, uint256 value)
    external returns (bool);

  /**
   * @dev Transfer tokens from one address to another
   * Selector: 23b872dd
   * @param from address The address which you want to send tokens from
   * @param to address The address which you want to transfer to
   * @param value uint256 the amount of tokens to be transferred
   * @return true if the transfer was succesful, revert otherwise.
   */
  function transferFrom(address from, address to, uint256 value)
    external returns (bool);

  /**
   * @dev Event emited when a transfer has been performed.
   * Selector: ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
   * @param from address The address sending the tokens
   * @param to address The address receiving the tokens.
   * @param value uint256 The amount of tokens transfered.
   */
  event Transfer(
    address indexed from,
    address indexed to,
    uint256 value
  );

  /**
   * @dev Event emited when an approval has been registered.
   * Selector: 8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925
   * @param owner address Owner of the tokens.
   * @param spender address Allowed spender.
   * @param value uint256 Amount of tokens approved.
   */
  event Approval(
    address indexed owner,
    address indexed spender,
    uint256 value
  );
}