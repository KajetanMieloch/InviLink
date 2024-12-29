from django.contrib import admin
from .models import UserProfile

class UserProfileAdmin(admin.ModelAdmin):
    list_display = ('user', 'nickname')
    search_fields = ('user__username', 'nickname')

admin.site.register(UserProfile, UserProfileAdmin)
