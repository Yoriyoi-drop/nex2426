# NEX2426 C API Documentation

## Overview

The NEX2426 C API provides C-compatible bindings for the NEX2426 encryption engine, enabling integration with C/C++ applications and other language bindings.

## Installation

### Header File

Include the header file in your C project:

```c
#include "nex2426.h"
```

The header file is located in the `include/` directory of the NEX2426 distribution.

### Library

Link against the NEX2426 library:

- **Static library**: `libnex2426.a`
- **Shared library**: `libnex2426.so` (Linux), `libnex2426.dylib` (macOS), `nex2426.dll` (Windows)

## Basic Usage

### Simple Hashing

```c
#include "nex2426.h"
#include <stdio.h>
#include <stdlib.h>

int main() {
    // Simple hash with default configuration
    char* hash = nex_hash_simple("Hello, NEX2426!", "my_key");
    if (hash) {
        printf("Hash: %s\n", hash);
        nex_free_string(hash);
    }
    return 0;
}
```

### Advanced Usage

```c
#include "nex2426.h"
#include <stdio.h>
#include <stdlib.h>

int main() {
    // Create configuration
    NexConfig config = NEX_CONFIG_INIT;
    config.cost = 5;
    config.temporal_binding = 1;
    
    // Create kernel
    NexKernel* kernel = nex_kernel_create(&config);
    if (!kernel) {
        fprintf(stderr, "Failed to create kernel\n");
        return 1;
    }
    
    // Hash data
    NexResult result = nex_hash_string(kernel, "test data", "test key");
    
    if (nex_result_is_success(&result)) {
        printf("Hash: %s\n", nex_result_get_hash(&result));
        printf("Timestamp: %lu\n", result.timestamp);
    } else {
        printf("Error: %s\n", nex_result_get_error(&result));
    }
    
    // Clean up
    nex_result_free(&result);
    nex_kernel_destroy(kernel);
    
    return 0;
}
```

## API Reference

### Data Structures

#### NexConfig

Configuration structure for kernel creation.

```c
typedef struct {
    uint32_t cost;              // Cost parameter (1-10)
    int temporal_binding;       // Enable temporal binding (0 = false, 1 = true)
    uint32_t reserved[8];       // Reserved for future use
} NexConfig;
```

#### NexResult

Result structure for API operations.

```c
typedef struct {
    int status;                 // Success status (0 = success, non-zero = error)
    char* hash;                 // Result hash string (null-terminated)
    char* error;                // Error message (null-terminated, null if no error)
    uint64_t timestamp;         // Timestamp from the operation
} NexResult;
```

#### NexKernel

Opaque handle for kernel instance.

```c
typedef struct NexKernel NexKernel;
```

### Constants

```c
#define NEX_DEFAULT_COST 3
#define NEX_DEFAULT_TEMPORAL_BINDING 0
#define NEX_MAX_COST 10

#define NEX_SUCCESS 0
#define NEX_ERROR_INVALID_PARAMS -1
#define NEX_ERROR_INVALID_ENCODING -2
#define NEX_ERROR_MEMORY_ALLOCATION -3
#define NEX_ERROR_INTERNAL -4
```

### Functions

#### Kernel Management

##### nex_kernel_create

```c
NexKernel* nex_kernel_create(const NexConfig* config);
```

Creates a new NEX2426 kernel instance.

**Parameters:**
- `config`: Configuration for the kernel

**Returns:**
- Pointer to kernel instance, or NULL on error

##### nex_kernel_destroy

```c
void nex_kernel_destroy(NexKernel* kernel);
```

Destroys a NEX2426 kernel instance.

**Parameters:**
- `kernel`: Pointer to kernel instance

#### Hashing Functions

##### nex_hash_data

```c
NexResult nex_hash_data(
    NexKernel* kernel,
    const uint8_t* data,
    size_t data_len,
    const char* key
);
```

Hash binary data using NEX2426 kernel.

**Parameters:**
- `kernel`: Pointer to kernel instance
- `data`: Pointer to input data
- `data_len`: Length of input data in bytes
- `key`: Pointer to encryption key (null-terminated string)

**Returns:**
- Result structure containing hash or error information

##### nex_hash_string

```c
NexResult nex_hash_string(
    NexKernel* kernel,
    const char* data,
    const char* key
);
```

Hash string data using NEX2426 kernel.

**Parameters:**
- `kernel`: Pointer to kernel instance
- `data`: Pointer to input string (null-terminated)
- `key`: Pointer to encryption key (null-terminated string)

**Returns:**
- Result structure containing hash or error information

#### Memory Management

##### nex_result_free

```c
void nex_result_free(NexResult* result);
```

Free result structure memory.

**Parameters:**
- `result`: Pointer to result structure

##### nex_free_string

```c
void nex_free_string(char* string);
```

Free string allocated by the library.

**Parameters:**
- `string`: Pointer to string (null-terminated)

#### Library Information

##### nex_get_version

```c
const char* nex_get_version(void);
```

Get library version information.

**Returns:**
- Pointer to version string (null-terminated)

##### nex_get_build_info

```c
const char* nex_get_build_info(void);
```

Get library build information.

**Returns:**
- Pointer to build info string (null-terminated)

#### Performance

##### nex_benchmark

```c
uint64_t nex_benchmark(
    NexKernel* kernel,
    uint32_t iterations,
    size_t data_size
);
```

Performance benchmark function.

**Parameters:**
- `kernel`: Pointer to kernel instance
- `iterations`: Number of iterations to run
- `data_size`: Size of test data in bytes

**Returns:**
- Average time per iteration in nanoseconds

### Utility Functions

#### Convenience Functions

##### nex_hash_simple

```c
char* nex_hash_simple(const char* data, const char* key);
```

Simple hash function with default configuration.

**Parameters:**
- `data`: Input string to hash
- `key`: Encryption key

**Returns:**
- Hash string (caller must free with nex_free_string), or NULL on error

##### nex_hash_with_cost

```c
char* nex_hash_with_cost(const char* data, const char* key, uint32_t cost);
```

Hash data with custom cost parameter.

**Parameters:**
- `data`: Input string to hash
- `key`: Encryption key
- `cost`: Cost parameter (1-10)

**Returns:**
- Hash string (caller must free with nex_free_string), or NULL on error

##### nex_hash_temporal

```c
char* nex_hash_temporal(const char* data, const char* key);
```

Hash data with temporal binding enabled.

**Parameters:**
- `data`: Input string to hash
- `key`: Encryption key

**Returns:**
- Hash string (caller must free with nex_free_string), or NULL on error

#### Error Handling

##### nex_result_is_success

```c
int nex_result_is_success(const NexResult* result);
```

Check if result indicates success.

**Parameters:**
- `result`: Result structure

**Returns:**
- 1 if successful, 0 otherwise

##### nex_result_get_error

```c
const char* nex_result_get_error(const NexResult* result);
```

Get error message from result.

**Parameters:**
- `result`: Result structure

**Returns:**
- Error message string, or NULL if no error

##### nex_result_get_hash

```c
const char* nex_result_get_hash(const NexResult* result);
```

Get hash from result.

**Parameters:**
- `result`: Result structure

**Returns:**
- Hash string, or NULL if no hash

## Error Handling

All functions return appropriate error codes and messages:

- `NEX_SUCCESS` (0): Operation successful
- `NEX_ERROR_INVALID_PARAMS` (-1): Invalid parameters
- `NEX_ERROR_INVALID_ENCODING` (-2): Invalid string encoding
- `NEX_ERROR_MEMORY_ALLOCATION` (-3): Memory allocation failed
- `NEX_ERROR_INTERNAL` (-4): Internal error

Always check the `status` field of `NexResult` structures and use `nex_result_get_error()` to retrieve error messages.

## Thread Safety

The NEX2426 C API is thread-safe with the following considerations:

- Each `NexKernel` instance can be used concurrently from multiple threads
- Different `NexKernel` instances can be used independently
- Library information functions (`nex_get_version`, `nex_get_build_info`) are thread-safe
- Memory management functions are thread-safe

## Performance Considerations

- Use appropriate cost parameters based on security requirements
- Higher cost parameters increase computation time
- Reuse kernel instances for multiple operations
- Use `nex_benchmark()` to measure performance on target hardware

## Memory Management

- All strings returned by the library must be freed using `nex_free_string()`
- `NexResult` structures must be freed using `nex_result_free()`
- Kernel instances must be destroyed using `nex_kernel_destroy()`
- Never free memory allocated by the library using standard `free()`

## Compilation

### GCC/Linux

```bash
gcc -o myapp myapp.c -lnex2426 -I/path/to/nex2426/include
```

### Clang/macOS

```bash
clang -o myapp myapp.c -lnex2426 -I/path/to/nex2426/include
```

### MSVC/Windows

```cmd
cl /I"path\to\nex2426\include" myapp.c nex2426.lib
```

## Troubleshooting

### Common Issues

1. **Linker errors**: Ensure the NEX2426 library is properly linked and accessible
2. **Header not found**: Verify the include path is correct
3. **Runtime crashes**: Check for NULL pointers and proper memory management
4. **Performance issues**: Adjust cost parameters and ensure proper kernel reuse

### Debug Mode

Compile with debug symbols for better error reporting:

```bash
gcc -g -o myapp myapp.c -lnex2426 -I/path/to/nex2426/include
```

## License

The NEX2426 C API is licensed under the same terms as the main NEX2426 project.
