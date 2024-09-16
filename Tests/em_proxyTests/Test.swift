//
//  Test.swift
//  em_proxy
//
//  Created by Joseph Mattiello on 9/16/24.
//

import Testing
@testable import em_proxy
@testable import libem_proxy

struct Test {

    @Test func test_em_proxy() async throws {

        var result: EMPError? = em_proxy.start_emotional_damage("127.0.0.1")
        #expect(result == nil)
        
        result = em_proxy.start_emotional_damage("127.0.0.1")
        
        #expect(result! == .AlreadyRunning)
        
        em_proxy.stop_emotional_damage()
    }
    
    @Test func test_em_proxy_test() async throws {
        var result: EMPError? = em_proxy.test_emotional_damage(10)
        
        #expect(result == nil)
    }
}
