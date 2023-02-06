# HuaHuaSwap CPAMM (Constant Product Automated Market Maker)

One such rule is the constant product formula B * Q = k, where B and Q are the reserves of the Base(B) and Quote(Q) token pair.

In order to withdraw some amount of Base tokens, one must deposit a proportional amount of Quote tokens to maintain the constant k before swap_fee_rate is applied and vice-versa.

&nbsp;

### How do we get initial LP shares T, after adding initial B and Q reserves?

f(B, Q) = sqrt(B * Q) = sqrt(k) => T

Why are we using the square_root of k
instead of just randomly choosing one of the numbers as the initial LP shares?

f(B, Q) is quadratic, meaning the bigger B and Q are, the larger the output; f(B, Q) should be linear.

&nbsp;

### Add liquidity: 

After adding liquidity, the pool price or slope before adding liquidity must be the same after adding liquidity.

Q / B = (Q + q) / (B + b)

Simlify by cross multiplying both sides

Q(B + b) = B(Q + q)

QB + Qb = BQ + Bq

Qb = Bq

Q / B = q / b

Therefore the ratio of current reserve (Q / B) must be equal to the ratio of added reserve (q / b).

&nbsp;

### How many LP shares to mint after adding liquidity?

The increases in LP shares is directly proportionally to increase in liquidity.

remember the function to calculate LP shares

f(B, Q) = sqrt(B * Q) = sqrt(k) => T

After adding liquidity

f((B + b), (Q + q)) = sqrt((B + b) * (Q + q)) => T'

Increase in Liquidity s is given below

s = T' - T

s = (q/Q)T = (b/B)T, where T = current LP shares

Verify that s = (q/Q)T ≡ (T' - T).

&nbsp;

### Remove Liquidity: How may tokens b and q to withdraw to burn s?

The decreases in LP shares is directly proportionally to decrease in liquidity.

b = (s/T)B and q = (s/T)Q where T' = T - s

&nbsp;

### How many tokens to return in a trade?

When you want to buy b from B tokens reserve, you pay by adding q to Q tokens reserve,
in other words, tokens are priced along an iso-liquidity price curve for non zero token reserves.

After a swap, the new reserves becomes

(B - b) * (Q + q) = k, where k = B * Q

Differentiate for output b

B - b = k / (Q + q)

b = B - (k / (Q + q))

b = B - (B * Q) / (Q + q)

b * (Q + q) = B * (Q + q) - (B * Q)

b * (Q + q) = BQ + Bq - BQ

b = Bq / (Q + q)

&nbsp;

### Price impact

This difference between the current market price and the expected fill price is called price impact.

Price impact is a function of
the size of your trade relative to the size of the liquidity pool.

price_impact = pool_price_after - pool_price_before , where pool_price = Q/B

&nbsp;

## How to test

See [TESTNET.md](https://github.com/ChihuahuaChain/Chiwawasm/blob/main/contracts/token-swap/TESTNET.md)

&nbsp;

## Future upgrades

Add support for stable swap

Add support for ranged liquidity pooling

&nbsp;

## References

[Paradigm research on price impact](https://research.paradigm.xyz/amm-price-impact#:~:text=One%20such%20rule%20is%20the,the%20constant%20k%20before%20fees)

[Explainer video on CPAMM](https://www.youtube.com/watch?v=QNPyFs8Wybk)

[JunoSwap AMM](https://junoswap.com/)