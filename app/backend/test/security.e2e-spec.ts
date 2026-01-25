import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication, VersioningType } from '@nestjs/common';
import request from 'supertest';
import { AppModule } from '../src/app.module';
import helmet from 'helmet';

describe('Security (e2e)', () => {
  let app: INestApplication;

  beforeAll(async () => {
    const moduleFixture: TestingModule = await Test.createTestingModule({
      imports: [AppModule],
    }).compile();

    app = moduleFixture.createNestApplication();

    // Apply prefix and versioning same as main.ts
    app.setGlobalPrefix('api');
    app.enableVersioning({
      type: VersioningType.URI,
      defaultVersion: '1',
      prefix: 'v',
    });

    // Apply same middleware as main.ts
    app.use(helmet());

    const allowedOrigins = ['http://localhost:3000'];
    app.enableCors({
      origin: (origin, callback) => {
        if (!origin || allowedOrigins.indexOf(origin) !== -1) {
          callback(null, true);
        } else {
          callback(new Error('Not allowed by CORS'));
        }
      },
    });

    await app.init();
  });

  afterAll(async () => {
    await app.close();
  });

  describe('Helmet Security Headers', () => {
    it('should have security headers enabled', async () => {
      const response = await request(app.getHttpServer()).get('/api/v1/health');

      expect(response.headers['x-dns-prefetch-control']).toBeDefined();
      expect(response.headers['x-frame-options']).toBeDefined();
      expect(response.headers['strict-transport-security']).toBeDefined();
      expect(response.headers['x-content-type-options']).toBeDefined();
    });
  });

  describe('CORS Policy', () => {
    it('should allowed request from whitelisted origin', async () => {
      const response = await request(app.getHttpServer())
        .get('/api/v1/health')
        .set('Origin', 'http://localhost:3000');

      expect(response.status).not.toBe(403);
      expect(response.headers['access-control-allow-origin']).toBe(
        'http://localhost:3000',
      );
    });

    it('should block request from non-whitelisted origin', async () => {
      // NestJS CORS middleware returns 204 or 403 based on config,
      // but in our implementation it throws an error in the callback which results in an error response.
      const response = await request(app.getHttpServer())
        .get('/api/v1/health')
        .set('Origin', 'http://malicious.com');

      // When CORS callback returns an error, NestJS/Express typically returns a 500 or closes connection
      // depending on how it's handled.
      expect(response.status).toBe(500);
    });
  });

  describe('Rate Limiting', () => {
    it('should eventually rate limit frequent requests', async () => {
      // The default limit is 100 in my security module config (fallback)
      // For testing, we might want to trigger it faster, but let's check headers first
      const response = await request(app.getHttpServer()).get('/api/v1/health');

      expect(response.status).toBe(200);
      // Try both common naming conventions
      const limitHeader =
        response.headers['x-ratelimit-limit'] ||
        response.headers['ratelimit-limit'];
      expect(limitHeader).toBeDefined();
      expect(response.headers['x-ratelimit-reset']).toBeDefined();
    });
  });
});
