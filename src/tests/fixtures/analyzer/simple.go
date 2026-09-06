// Simple 3-round ping-pong implementation per specification
package main

import (
	"fmt"
	"sync"
	"time"
)

type SimpleMessage struct {
	Type  string
	Round int
}

func SimplePingPong(pingChan, pongChan chan SimpleMessage) []SimpleMessage {
	var messages []SimpleMessage
	var wg sync.WaitGroup
	messageMutex := &sync.Mutex{}

	// Ping actor
	wg.Add(1)
	go func() {
		defer wg.Done()
		for round := 1; round <= 3; round++ {
			// Send ping
			pingMsg := SimpleMessage{Type: "ping", Round: round}
			fmt.Printf("Ping: Sending round %d\n", round)
			pongChan <- pingMsg

			messageMutex.Lock()
			messages = append(messages, pingMsg)
			messageMutex.Unlock()

			// Wait for pong
			select {
			case pongMsg := <-pingChan:
				fmt.Printf("Ping: Received pong %d\n", pongMsg.Round)
				messageMutex.Lock()
				messages = append(messages, pongMsg)
				messageMutex.Unlock()
			case <-time.After(5 * time.Millisecond):
				fmt.Printf("Ping: Timeout waiting for pong\n")
			}
		}
	}()

	// Pong actor
	wg.Add(1)
	go func() {
		defer wg.Done()
		for round := 1; round <= 3; round++ {
			// Receive ping
			select {
			case pingMsg := <-pongChan:
				fmt.Printf("Pong: Received ping %d\n", pingMsg.Round)

				// Send pong back
				pongMsg := SimpleMessage{Type: "pong", Round: pingMsg.Round}
				pingChan <- pongMsg
				fmt.Printf("Pong: Sent pong %d\n", pongMsg.Round)
			case <-time.After(5 * time.Millisecond):
				fmt.Printf("Pong: Timeout waiting for ping\n")
			}
		}
	}()

	wg.Wait()
	return messages
}
