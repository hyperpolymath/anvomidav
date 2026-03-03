# SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
# SPDX-License-Identifier: PMPL-1.0-or-later

"""
    AnvomidavPkg

Package manager for Anvomidav figure skating DSL.

Provides functionality to:
- Install Anvomidav packages (choreographies, elements, notation libraries)
- Search the Anvomidav package registry
- Resolve dependencies
- Update installed packages
- Manage skating notation libraries

# Examples

```julia
using AnvomidavPkg

# Install a choreography package
AnvomidavPkg.install("classic-programs")

# Search for packages
AnvomidavPkg.search("olympic")

# Update dependencies
AnvomidavPkg.resolve()

# List installed packages
AnvomidavPkg.list()
```
"""
module AnvomidavPkg

using HTTP
using JSON3
using TOML

export install, search, resolve, update, list, info

const REGISTRY_URL = "https://registry.anvomidav.org"
const CACHE_DIR = joinpath(homedir(), ".anvomidav", "cache")
const PACKAGES_DIR = joinpath(homedir(), ".anvomidav", "packages")

"""
    install(package_name::String; version::String="latest")

Install an Anvomidav package from the registry.

# Arguments
- `package_name::String`: Name of the package to install
- `version::String`: Version to install (default: "latest")

# Examples
```julia
AnvomidavPkg.install("classic-programs")
AnvomidavPkg.install("isu-elements", version="2.0.0")
```
"""
function install(package_name::String; version::String="latest")
    println("📦 Installing $package_name...")

    # Create directories if they don't exist
    mkpath(CACHE_DIR)
    mkpath(PACKAGES_DIR)

    # Fetch package metadata
    registry = fetch_registry()

    if !haskey(registry, package_name)
        error("Package '$package_name' not found in registry")
    end

    pkg_info = registry[package_name]

    # Resolve version
    resolved_version = version == "latest" ? pkg_info["latest"] : version

    if !haskey(pkg_info["versions"], resolved_version)
        error("Version '$resolved_version' not found for package '$package_name'")
    end

    version_info = pkg_info["versions"][resolved_version]

    # Download package
    url = version_info["url"]
    dest = joinpath(PACKAGES_DIR, "$package_name-$resolved_version")

    println("  Downloading from $url...")
    HTTP.download(url, dest)

    # Extract (if tarball)
    if endswith(url, ".tar.gz")
        run(`tar -xzf $dest -C $PACKAGES_DIR`)
        rm(dest)
    end

    # Install dependencies
    if haskey(version_info, "dependencies")
        for (dep_name, dep_version) in version_info["dependencies"]
            println("  Installing dependency: $dep_name@$dep_version")
            install(dep_name, version=dep_version)
        end
    end

    println("✓ Successfully installed $package_name@$resolved_version")

    return nothing
end

"""
    search(query::String; limit::Int=10)

Search for packages in the Anvomidav registry.

# Arguments
- `query::String`: Search query (matches package names and descriptions)
- `limit::Int`: Maximum number of results to return (default: 10)

# Examples
```julia
AnvomidavPkg.search("olympic")
AnvomidavPkg.search("jump", limit=5)
```
"""
function search(query::String; limit::Int=10)
    println("🔍 Searching for '$query'...")

    registry = fetch_registry()
    results = []

    query_lower = lowercase(query)

    for (name, info) in registry
        name_lower = lowercase(name)
        desc_lower = lowercase(get(info, "description", ""))

        if occursin(query_lower, name_lower) || occursin(query_lower, desc_lower)
            push!(results, (name, info))
        end

        if length(results) >= limit
            break
        end
    end

    if isempty(results)
        println("No packages found matching '$query'")
        return nothing
    end

    println("\nFound $(length(results)) package(s):\n")

    for (name, info) in results
        latest = info["latest"]
        desc = get(info, "description", "No description")
        println("  $name ($latest)")
        println("    $desc\n")
    end

    return nothing
end

"""
    resolve()

Resolve and install all dependencies for the current project.

Reads the Anvomidav.toml file in the current directory and ensures
all dependencies are installed at compatible versions.

# Examples
```julia
AnvomidavPkg.resolve()
```
"""
function resolve()
    println("🔧 Resolving dependencies...")

    if !isfile("Anvomidav.toml")
        error("No Anvomidav.toml found in current directory")
    end

    manifest = TOML.parsefile("Anvomidav.toml")

    if !haskey(manifest, "dependencies")
        println("No dependencies specified")
        return nothing
    end

    deps = manifest["dependencies"]

    for (name, version) in deps
        println("  Resolving $name@$version")
        install(name, version=version)
    end

    println("✓ All dependencies resolved")

    return nothing
end

"""
    update(package_name::String="all")

Update one or all installed packages to their latest versions.

# Arguments
- `package_name::String`: Package to update, or "all" for all packages

# Examples
```julia
AnvomidavPkg.update()  # Update all
AnvomidavPkg.update("classic-programs")  # Update specific package
```
"""
function update(package_name::String="all")
    println("⬆️  Updating packages...")

    installed = list_installed()

    if isempty(installed)
        println("No packages installed")
        return nothing
    end

    if package_name == "all"
        for pkg in installed
            install(pkg["name"], version="latest")
        end
    else
        install(package_name, version="latest")
    end

    println("✓ Update complete")

    return nothing
end

"""
    list()

List all installed Anvomidav packages.

# Examples
```julia
AnvomidavPkg.list()
```
"""
function list()
    installed = list_installed()

    if isempty(installed)
        println("No packages installed")
        return nothing
    end

    println("Installed packages:\n")

    for pkg in installed
        println("  $(pkg["name"]) ($(pkg["version"]))")
    end

    println("\nTotal: $(length(installed)) package(s)")

    return nothing
end

"""
    info(package_name::String)

Show detailed information about a package.

# Arguments
- `package_name::String`: Name of the package

# Examples
```julia
AnvomidavPkg.info("classic-programs")
```
"""
function info(package_name::String)
    registry = fetch_registry()

    if !haskey(registry, package_name)
        error("Package '$package_name' not found")
    end

    pkg = registry[package_name]

    println("Package: $package_name")
    println("Latest version: $(pkg["latest"])")
    println("Description: $(get(pkg, "description", "No description"))")
    println("Repository: $(get(pkg, "repository", "N/A"))")
    println()

    println("Available versions:")
    for (version, info) in pkg["versions"]
        println("  $version (released: $(info["released"]))")
    end

    return nothing
end

# Internal functions

function fetch_registry()
    cache_file = joinpath(CACHE_DIR, "registry.json")

    # Use cached registry if available and recent (< 1 hour old)
    if isfile(cache_file) && (time() - mtime(cache_file)) < 3600
        return JSON3.read(read(cache_file, String))
    end

    # Fetch from remote
    mkpath(CACHE_DIR)

    try
        response = HTTP.get("$REGISTRY_URL/registry.json")
        registry_json = String(response.body)

        # Cache for next time
        write(cache_file, registry_json)

        return JSON3.read(registry_json)
    catch e
        # Fallback to cached version if network fails
        if isfile(cache_file)
            @warn "Failed to fetch registry, using cached version"
            return JSON3.read(read(cache_file, String))
        else
            error("Failed to fetch registry and no cache available: $e")
        end
    end
end

function list_installed()
    if !isdir(PACKAGES_DIR)
        return []
    end

    installed = []

    for entry in readdir(PACKAGES_DIR)
        if isdir(joinpath(PACKAGES_DIR, entry))
            parts = split(entry, '-')
            if length(parts) >= 2
                name = join(parts[1:end-1], '-')
                version = parts[end]
                push!(installed, Dict("name" => name, "version" => version))
            end
        end
    end

    return installed
end

end # module
