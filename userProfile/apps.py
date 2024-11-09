from django.apps import AppConfig

class UserProfileConfig(AppConfig):
    name = 'userProfile'

    def ready(self):
        import userProfile.signals
