cmake_minimum_required(VERSION 3.16)
project(Registry VERSION 1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 REQUIRED COMPONENTS Core Gui Widgets Qml Quick QuickWidgets Concurrent)

# ── FFI shared library (built by `make ffi`) ──────────────────────────────────
add_library(registry_ffi SHARED IMPORTED)
set_target_properties(registry_ffi PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/../../target/debug/libregistry_ffi.so"
)

# ── Plugin (loaded by Basecamp) ────────────────────────────────────────────
add_library(RegistryPlugin SHARED
    src/RegistryBackend.cpp
    src/RegistryPlugin.cpp
)

# Resource name "registry_qml" must match Q_INIT_RESOURCE(registry_qml) in RegistryPlugin.cpp
qt_add_resources(RegistryPlugin "registry_qml"
    PREFIX "/"
    FILES qml/Main.qml
)

target_link_libraries(RegistryPlugin PRIVATE
    Qt6::Core Qt6::Gui Qt6::Widgets Qt6::Qml Qt6::Quick Qt6::QuickWidgets Qt6::Concurrent
    registry_ffi
)

# Output name must match manifest.json "main" value (lib<name>_plugin.so)
set_target_properties(RegistryPlugin PROPERTIES OUTPUT_NAME "registry_plugin")

# ── Standalone preview app ─────────────────────────────────────────────────
add_executable(RegistryApp
    src/main.cpp
    src/RegistryBackend.cpp
    src/RegistryPlugin.cpp
)

# Same resource name as the plugin so Q_INIT_RESOURCE resolves in both targets
qt_add_resources(RegistryApp "registry_qml"
    PREFIX "/"
    FILES qml/Main.qml
)

target_link_libraries(RegistryApp PRIVATE
    Qt6::Core Qt6::Gui Qt6::Widgets Qt6::Qml Qt6::Quick Qt6::QuickWidgets Qt6::Concurrent
    registry_ffi
)

set_target_properties(RegistryApp PROPERTIES
    BUILD_RPATH "${CMAKE_CURRENT_SOURCE_DIR}/../../target/debug"
)
