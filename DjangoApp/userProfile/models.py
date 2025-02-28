from django.db import models
from django.contrib.auth.models import User

class UserProfile(models.Model):
    user = models.OneToOneField(User, on_delete=models.CASCADE)
    nickname = models.CharField(max_length=100, blank=True, null=True)
    public_key = models.CharField(max_length=150, blank=True, null=True)

    def __str__(self):
        return self.user.username