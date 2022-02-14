*** Settings ***
Suite Setup                   Setup
Suite Teardown                Teardown
Test Setup                    Reset Emulation
Resource                      ${RENODEKEYWORDS}

*** Test Cases ***
Test Button Led Toggle
    Execute Command         mach create
    Execute Command         machine LoadPlatformDescription @${CURDIR}/nrf52840-dongle.repl
    Execute Command         sysbus LoadELF @${CURDIR}/../firmware/target/thumbv7em-none-eabi/debug/frenode

    Create Terminal Tester  sysbus.uart0
    Execute Command         emulation CreateLEDTester "lt" sysbus.gpio0.led

    Start Emulation

    Wait For Line On Uart   Hello World!
    Execute Command         lt AssertState False 0
    Execute Command         sysbus.gpioPortA.UserButton Press
    Test If Uart Is Idle    3
    Execute Command         lt AssertState True 0
    Execute Command         sysbus.gpioPortA.UserButton Release
    Test If Uart Is Idle    3
    Execute Command         lt AssertState False 0
