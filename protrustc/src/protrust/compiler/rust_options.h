#ifndef PROTRUSTC_RUST_OPTIONS_H__
#define PROTRUSTC_RUST_OPTIONS_H__

#include <string>
#include <vector>

namespace protrust {
namespace compiler {

struct Options {
    Options() :
        file_extension(".rs") {

    }
    
    std::string file_extension;
    std::vector<std::string> imports;
};

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_OPTIONS_H__