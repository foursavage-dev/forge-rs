package network

import (
	"bytes"
	"sync"
)

type BufferPool struct {
	pool sync.Pool
	size int
}

func NewBufferPool(bufferSize int) *BufferPool {
	return &BufferPool{
		size: bufferSize,
		pool: sync.Pool{
			New: func() interface{} {
				return bytes.NewBuffer(make([]byte, 0, bufferSize))
			},
		},
	}
}

func (p *BufferPool) Get() *bytes.Buffer {
	buf := p.pool.Get().(*bytes.Buffer)
	buf.Reset()
	return buf
}

func (p *BufferPool) Put(buf *bytes.Buffer) {
	if buf.Cap() > p.size*4 {
		return
	}
	p.pool.Put(buf)
}
