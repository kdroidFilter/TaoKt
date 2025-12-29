package io.github.kdroidfilter.taokt.examples

fun main(args: Array<String>) {
    val name = args.firstOrNull()
    if (name == null || name in setOf("list", "--list", "-l", "help", "--help", "-h")) {
        Examples.printHelp()
        return
    }

    val runner = Examples.ALL[name]
    if (runner == null) {
        println("Unknown example: $name")
        Examples.printHelp()
        return
    }

    runner(args.drop(1))
}
