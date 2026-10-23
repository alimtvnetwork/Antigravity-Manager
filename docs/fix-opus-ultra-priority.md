# Fix Opus 4.6 call error & UserToken display optimization

## question

The account pool is a mix of Pro and Ultra accounts. Pro doesn't have Opus 4.6 access, Ultra does.

Previously, polling selected accounts based on quotas, regardless of subscription level. When users adjust Opus 4.6, the system may select the Pro account and directly report an error.

## change

### 1. Ultra priority scheduling

When adjusting Opus 4.6/4.5, first sort by subscription level:

```
Ultra > Pro > Free
```

Those at the same level will be ranked according to quota. Other models still use the old logic, with quota priority.

Matching rules: If the model name contains `claude-opus-4-6`, `claude-opus-4-5` or `opus`, Ultra will be given priority.

### 2. UserToken editing data is not displayed

When clicking Edit Token, the IP restrictions and curfew times are empty.

question:
- The front-end uses `undefined` to pass parameters, and Rust requires `null`
- Use `||` to read, 0 and empty strings are eaten, changed to `??`

### 3. Customize expiration time

There is an additional Custom option when creating a Token. Select the date and time, accurate to the hour.

## document

```
src-tauri/src/proxy/token_manager.rs # Sorting logic
src-tauri/src/proxy/tests/mod.rs # Test module
src-tauri/src/proxy/tests/ultra_priority_tests.rs # Ultra priority testing
src-tauri/src/commands/user_token.rs # Custom expiration parameters
src-tauri/src/modules/user_token_db.rs # Database
src/pages/UserToken.tsx # Front-end
```

## verify

1. Adjust Opus 4.6 and check the logs to confirm that you are using an Ultra account.
2. Create Token, set IP restrictions and curfew, and confirm that the data is echoed normally when editing.
