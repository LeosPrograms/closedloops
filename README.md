# MTCS Rust

Rust implementation of Multilateral Trade Credit Set-off algorithm

## Input

Input file in CSV format in [data](./data/) folder.

Example input

```
id,debtor,creditor,amount
1,0,65,136
2,0,167,183
3,0,1224,467
4,0,1328,1200
5,0,1748,6250
```

Where id is an obligation ID

## Output

Output file in CVS format in [result](./result/) folder

Sample output

```
id,amount
1,0
2,183
3,467
4,0
5,2310
```

For every obligation we state the setoff amount.

## Terminal dump

Successful run wil produce the following terminal dump

```
----------------------------------
            NID = 54
     Total debt = 104
Total remainder = 68
  Total cleared = 36
```

NID is Net Internal Debt

Algorithm fails if the sums don't add up

## Use

```
cargo run file.csv