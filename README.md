# N2O

This project is designed for super fast in-memory lookup of numbers remotely, converting them to 10-digit format for storage. It supports up to two “sender” values associated with each phone number (i.e., 2 different phone numbers from which we have texted the same user).

Endpoints planned:
1. add
2. addmulti
3. dump
4. clear
5. status

Deployment details:
- Start on port 1337; automatically switch to 1338 if 1337 is taken.  
- Requires token-based authentication.  
- .env file will list valid tokens.

Possible use-case: avoid sending multiple SMS to the same number from more than two of our numbers.
