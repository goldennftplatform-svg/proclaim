/* Auto-generated C header for registry FFI. DO NOT EDIT. */
#ifndef REGISTRY_FFI_H
#define REGISTRY_FFI_H

#ifdef __cplusplus
extern "C" {
#endif

/* init_index instruction */
char* registry_init_index(const char* args_json);

/* register instruction */
char* registry_register(const char* args_json);

/* update instruction */
char* registry_update(const char* args_json);

/* mark_verified instruction */
char* registry_mark_verified(const char* args_json);

/* lookup instruction */
char* registry_lookup(const char* args_json);

/* search instruction */
char* registry_search(const char* args_json);

/* fetch index account state */
char* registry_fetch_index(const char* args_json);

/* fetch entry account state */
char* registry_fetch_entry(const char* args_json);

/* wallet helpers */
char* registry_check_connection(const char* args_json);
char* registry_inspect_account(const char* args_json);
char* registry_list_accounts(const char* args_json);
char* registry_create_account(const char* args_json);

void registry_free_string(char* s);
char* registry_version(void);
char* registry_program_id(void);
char* registry_decode_account(const char* args_json);

#ifdef __cplusplus
}
#endif

#endif /* REGISTRY_FFI_H */
