from odoo import models, api
import secrets

class ResConfigSettings(models.TransientModel):
    _inherit = 'res.config.settings'

    def set_rust_webhook_token(self):
        # Genera un token seguro de 32 caracteres si no existe
        existing_token = self.env['ir.config_parameter'].sudo().get_param('rust_api.webhook_token')
        if not existing_token:
            new_token = secrets.token_urlsafe(32)
            self.env['ir.config_parameter'].sudo().set_param('rust_api.webhook_token', new_token)