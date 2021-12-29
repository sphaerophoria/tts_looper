project(gui CXX)

add_compile_options(-Wall -Wextra -Wpedantic)
find_package(Qt5 COMPONENTS Quick REQUIRED)
add_library(gui gui.cpp res/res.qrc)
target_link_libraries(gui PUBLIC Qt5::Quick)
set_target_properties(gui PROPERTIES AUTOMOC true AUTORCC true)
target_compile_features(gui PRIVATE cxx_std_17)

install(TARGETS gui)
