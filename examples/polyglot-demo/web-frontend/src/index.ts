interface ServiceResponse {
    message: string;
    timestamp: number;
}

class PolyglotClient {
    private services: string[] = [
        'http://localhost:8080',
        'http://localhost:8081'
    ];

    async callAllServices(): Promise<ServiceResponse[]> {
        const responses: ServiceResponse[] = [];
        
        for (const service of this.services) {
            try {
                const response = await fetch(service);
                const data = await response.text();
                responses.push({
                    message: data,
                    timestamp: Date.now()
                });
            } catch (error) {
                console.error(`Failed to call ${service}:`, error);
            }
        }
        
        return responses;
    }
}

async function main() {
    console.log('🌐 TypeScript Frontend starting...');
    
    const client = new PolyglotClient();
    const responses = await client.callAllServices();
    
    console.log('Service responses:', responses);
}

main().catch(console.error);
