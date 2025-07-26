#!/usr/bin/env node

/**
 * Keygen Product Setup Script for AI Memory Engine
 * This script creates a new product and policy via Keygen API
 */

const https = require('https');
const fs = require('fs');

const ACCOUNT_ID = 'c86687d0-695b-474c-bd18-e37d96969dcb';

// You'll need to get this from your Keygen dashboard
const ADMIN_TOKEN = process.env.KEYGEN_ADMIN_TOKEN || '';

if (!ADMIN_TOKEN) {
    console.error('❌ Please set KEYGEN_ADMIN_TOKEN environment variable');
    console.error('   Get this from: https://app.keygen.sh/settings/tokens');
    process.exit(1);
}

function makeRequest(method, path, data = null) {
    return new Promise((resolve, reject) => {
        const options = {
            hostname: 'api.keygen.sh',
            port: 443,
            path: `/v1/accounts/${ACCOUNT_ID}${path}`,
            method: method,
            headers: {
                'Authorization': `Bearer ${ADMIN_TOKEN}`,
                'Content-Type': 'application/vnd.api+json',
                'Accept': 'application/vnd.api+json'
            }
        };

        if (data) {
            const jsonData = JSON.stringify(data);
            options.headers['Content-Length'] = Buffer.byteLength(jsonData);
        }

        const req = https.request(options, (res) => {
            let responseData = '';
            
            res.on('data', (chunk) => {
                responseData += chunk;
            });
            
            res.on('end', () => {
                try {
                    const parsed = JSON.parse(responseData);
                    if (res.statusCode >= 200 && res.statusCode < 300) {
                        resolve(parsed);
                    } else {
                        reject(new Error(`HTTP ${res.statusCode}: ${JSON.stringify(parsed)}`));
                    }
                } catch (e) {
                    reject(new Error(`Parse error: ${e.message}`));
                }
            });
        });

        req.on('error', (err) => {
            reject(err);
        });

        if (data) {
            req.write(JSON.stringify(data));
        }

        req.end();
    });
}

async function createProduct() {
    console.log('🚀 Creating AI Memory Engine product...');
    
    const productData = {
        data: {
            type: 'products',
            attributes: {
                name: 'AI Memory Engine',
                url: 'https://github.com/jahboukie/aimemoryengine',
                distributionStrategy: 'LICENSED', // Requires license for access
                platforms: ['win32', 'darwin', 'linux']
            }
        }
    };

    try {
        const response = await makeRequest('POST', '/products', productData);
        console.log('✅ Product created successfully!');
        console.log(`   Product ID: ${response.data.id}`);
        return response.data;
    } catch (error) {
        console.error('❌ Failed to create product:', error.message);
        throw error;
    }
}

async function createPolicies(productId) {
    console.log('🔐 Creating license policies...');

    const policies = [
        {
            name: 'Individual License',
            maxMachines: 1,
            price: '$49/year',
            monthly: '$4.08/month',
            tier: 'individual',
            description: 'Perfect for solo developers and personal projects'
        },
        {
            name: 'Professional License',
            maxMachines: 3,
            price: '$99/year',
            monthly: '$8.25/month',
            tier: 'professional',
            description: 'Ideal for professional developers with multiple machines'
        },
        {
            name: 'Team License',
            maxMachines: 10,
            price: '$199/year',
            monthly: '$16.58/month',
            tier: 'team',
            description: 'Great for small teams and development groups'
        },
        {
            name: 'Enterprise License',
            maxMachines: null, // Unlimited
            price: '$499/year',
            monthly: '$41.58/month',
            tier: 'enterprise',
            description: 'Unlimited machines for large organizations'
        }
    ];

    const createdPolicies = [];

    for (const policy of policies) {
        console.log(`   Creating ${policy.name}...`);

        const policyData = {
            data: {
                type: 'policies',
                attributes: {
                    name: policy.name,
                    duration: 31536000, // 1 year in seconds
                    strict: true,
                    floating: false,
                    scheme: 'ED25519_SIGN',
                    requireHeartbeat: false,
                    heartbeatDuration: null,
                    heartbeatCullStrategy: 'DEACTIVATE_DEAD',
                    heartbeatResurrectionStrategy: 'NO_REVIVE',
                    heartbeatBasis: 'FROM_CREATION',
                    machineUniquenessStrategy: 'UNIQUE_PER_MACHINE',
                    machineMatchingStrategy: 'MATCH_ANY',
                    componentMatchingStrategy: 'MATCH_ANY',
                    expirationStrategy: 'RESTRICT_ACCESS',
                    expirationBasis: 'FROM_CREATION',
                    renewalBasis: 'FROM_EXPIRY',
                    transferStrategy: 'KEEP_EXPIRY',
                    authenticationStrategy: 'TOKEN',
                    machineLeasingStrategy: 'PER_LICENSE',
                    processLeasingStrategy: 'PER_MACHINE',
                    overageStrategy: 'NO_OVERAGE',
                    requireCheckIn: true,
                    checkInInterval: 'month',
                    maxMachines: policy.maxMachines,
                    metadata: {
                        description: policy.description,
                        price: policy.price,
                        monthly_equivalent: policy.monthly,
                        tier: policy.tier,
                        machines: policy.maxMachines ? `${policy.maxMachines} machines` : 'Unlimited machines',
                        features: [
                            'persistent_memory',
                            'multi_language_support',
                            'query_engine',
                            policy.tier === 'individual' ? 'community_support' : 'priority_support',
                            ...(policy.tier === 'enterprise' ? ['dedicated_support', 'custom_integrations'] : [])
                        ]
                    }
                },
                relationships: {
                    product: {
                        data: {
                            type: 'products',
                            id: productId
                        }
                    }
                }
            }
        };

        try {
            const response = await makeRequest('POST', '/policies', policyData);
            console.log(`   ✅ ${policy.name} created - ${policy.price}`);
            createdPolicies.push(response.data);
        } catch (error) {
            console.error(`   ❌ Failed to create ${policy.name}:`, error.message);
            throw error;
        }
    }

    return createdPolicies;
}

async function createApiToken(productId) {
    console.log('🔑 Creating API token...');
    
    const tokenData = {
        data: {
            type: 'tokens',
            attributes: {
                name: 'AI Memory Engine API Token',
                permissions: ['license.validate', 'license.read']
            },
            relationships: {
                product: {
                    data: {
                        type: 'products',
                        id: productId
                    }
                }
            }
        }
    };

    try {
        const response = await makeRequest('POST', '/tokens', tokenData);
        console.log('✅ API token created successfully!');
        console.log(`   Token: ${response.data.attributes.token}`);
        return response.data;
    } catch (error) {
        console.error('❌ Failed to create API token:', error.message);
        throw error;
    }
}

async function updateEnvFile(productId, apiToken) {
    console.log('📝 Updating .env file...');
    
    const envContent = `KEYGEN_API_KEY=${apiToken}
KEYGEN_PRODUCT_ID=${productId}
KEYGEN_ACCOUNT_ID=${ACCOUNT_ID}
`;

    try {
        fs.writeFileSync('.env', envContent);
        console.log('✅ .env file updated successfully!');
        console.log('   File: .env');
    } catch (error) {
        console.error('❌ Failed to update .env file:', error.message);
        console.log('📋 Manual setup required:');
        console.log(envContent);
    }
}

async function main() {
    try {
        console.log('🧠 AI Memory Engine - Keygen Setup');
        console.log('=====================================');
        
        // Create product
        const product = await createProduct();

        // Create all pricing tier policies
        const policies = await createPolicies(product.id);

        // Create API token
        const token = await createApiToken(product.id);

        // Update .env file
        await updateEnvFile(product.id, token.attributes.token);

        console.log('\n🎉 Setup complete!');
        console.log('=====================================');
        console.log(`Product ID: ${product.id}`);
        console.log(`Policies Created: ${policies.length}`);
        policies.forEach(policy => {
            console.log(`  - ${policy.attributes.name}: ${policy.id}`);
        });
        console.log(`API Token: ${token.attributes.token.substring(0, 20)}...`);
        console.log('\n📋 Next steps:');
        console.log('1. Test license validation with: cargo run --bin aimemoryengine -- license status');
        console.log('2. Create test licenses in Keygen dashboard');
        console.log('3. Test license activation with: cargo run --bin aimemoryengine -- license activate <key>');
        
    } catch (error) {
        console.error('\n❌ Setup failed:', error.message);
        console.log('\n📋 Manual setup required:');
        console.log('1. Go to https://app.keygen.sh');
        console.log('2. Create new product: "AI Memory Engine"');
        console.log('3. Create license policy');
        console.log('4. Generate API token');
        console.log('5. Update .env file with credentials');
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}
